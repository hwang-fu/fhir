//! AI chatbot with tool calling for FHIR data queries

use super::client::{ClaudeClient, Content, ContentBlock, Message, Tool};
use crate::db::PatientRepository;
use serde_json::{Value as JsonValue, json};

const SYSTEM_PROMPT: &str = r#"You are a helpful FHIR Patient data assistant. You can search for patients, retrieve specific patient records, and count patients in the system.

Use the available tools to answer questions about patient data. Always use tools when you need to look up data — don't guess or make up patient information.

When presenting results, format them clearly and concisely. Include relevant details like patient names, gender, and birth dates."#;

/// Maximum agentic loop iterations to prevent runaway
const MAX_ITERATIONS: u32 = 10;

/// Define the tools available to the chatbot
fn chat_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "search_patients".to_string(),
            description: "Search for patients using FHIR search parameters".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Patient name to search for (partial match)"
                    },
                    "gender": {
                        "type": "string",
                        "enum": ["male", "female", "other", "unknown"],
                        "description": "Patient gender"
                    },
                    "birthdate": {
                        "type": "string",
                        "description": "Birth date with optional FHIR prefix (e.g. ge1990-01-01)"
                    }
                },
                "additionalProperties": false
            }),
        },
        Tool {
            name: "get_patient".to_string(),
            description: "Get a specific patient by their UUID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The patient's UUID"
                    }
                },
                "required": ["id"],
                "additionalProperties": false
            }),
        },
        Tool {
            name: "count_patients".to_string(),
            description: "Count the total number of patients matching search criteria".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Patient name to filter by (optional)"
                    },
                    "gender": {
                        "type": "string",
                        "enum": ["male", "female", "other", "unknown"],
                        "description": "Patient gender to filter by (optional)"
                    },
                    "birthdate": {
                        "type": "string",
                        "description": "Birth date filter with FHIR prefix (optional)"
                    }
                },
                "additionalProperties": false
            }),
        },
    ]
}

/// Build a FHIR search params JSON object from tool input
fn build_search_params(input: &JsonValue) -> JsonValue {
    let mut params = serde_json::Map::new();
    if let Some(name) = input.get("name").and_then(|v| v.as_str()) {
        params.insert("name".to_string(), json!(name));
    }
    if let Some(gender) = input.get("gender").and_then(|v| v.as_str()) {
        params.insert("gender".to_string(), json!(gender));
    }
    if let Some(bd) = input.get("birthdate").and_then(|v| v.as_str()) {
        params.insert("birthdate".to_string(), json!(bd));
    }
    JsonValue::Object(params)
}

/// Execute a tool call against the database
async fn execute_tool(repo: &PatientRepository, name: &str, input: &JsonValue) -> String {
    match name {
        "search_patients" => {
            let mut params = build_search_params(input);
            // Limit results to avoid huge responses sent back to Claude
            if let Some(obj) = params.as_object_mut() {
                obj.insert("_count".to_string(), json!(20));
            }

            match repo.search(params).await {
                Ok(results) => {
                    let patients: Vec<JsonValue> = results
                        .into_iter()
                        .map(|(id, data)| json!({"id": id.to_string(), "resource": data}))
                        .collect();
                    serde_json::to_string(&patients).unwrap_or_else(|_| "[]".to_string())
                }
                Err(e) => format!("Error searching patients: {e:?}"),
            }
        }
        "get_patient" => {
            let id_str = input.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match uuid::Uuid::parse_str(id_str) {
                Ok(id) => match repo.get(id).await {
                    Ok(Some(data)) => {
                        serde_json::to_string(&data).unwrap_or_else(|_| "null".to_string())
                    }
                    Ok(None) => format!("Patient {id} not found"),
                    Err(e) => format!("Error getting patient: {e:?}"),
                },
                Err(_) => format!("Invalid UUID: {id_str}"),
            }
        }
        "count_patients" => {
            let params = build_search_params(input);
            match repo.count(params).await {
                Ok(count) => format!("{count}"),
                Err(e) => format!("Error counting patients: {e:?}"),
            }
        }
        _ => format!("Unknown tool: {name}"),
    }
}

/// Run the chatbot agentic loop.
///
/// Sends the user message to Claude with tools, executes any tool calls,
/// and continues until Claude produces a final text response.
pub async fn chat(
    client: &ClaudeClient,
    repo: &PatientRepository,
    user_message: &str,
) -> Result<String, String> {
    let tools = chat_tools();

    let mut messages = vec![Message {
        role: "user".to_string(),
        content: Content::Text(user_message.to_string()),
    }];

    for iteration in 0..MAX_ITERATIONS {
        let response = client
            .send(Some(SYSTEM_PROMPT), messages.clone(), Some(tools.clone()))
            .await?;

        tracing::debug!(
            iteration = iteration,
            stop_reason = &response.stop_reason,
            "Chat loop iteration"
        );

        // If Claude is done talking, return the text
        if response.stop_reason == "end_turn" {
            return client.extract_text(&response);
        }

        if response.stop_reason == "tool_use" {
            // Collect tool_use blocks
            let tool_uses: Vec<_> = response
                .content
                .iter()
                .filter_map(|block| {
                    if let ContentBlock::ToolUse { id, name, input } = block {
                        Some((id.clone(), name.clone(), input.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            // Append the assistant's response (with tool_use blocks) to conversation
            messages.push(Message {
                role: "assistant".to_string(),
                content: Content::Blocks(response.content),
            });

            // Execute each tool and collect results
            let mut result_blocks = Vec::new();
            for (tool_id, tool_name, tool_input) in &tool_uses {
                tracing::info!(tool = %tool_name, "Executing chat tool");
                let result = execute_tool(repo, tool_name, tool_input).await;
                result_blocks.push(ContentBlock::ToolResult {
                    tool_use_id: tool_id.clone(),
                    content: result,
                });
            }

            // Send tool results back as a user message
            messages.push(Message {
                role: "user".to_string(),
                content: Content::Blocks(result_blocks),
            });
        } else {
            // Unexpected stop reason — return whatever text we got
            return client.extract_text(&response);
        }
    }

    Err("Chat loop exceeded maximum iterations".to_string())
}

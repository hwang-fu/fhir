//! Claude API client for the Anthropic Messages API

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";

/// Client for the Anthropic Claude Messages API
#[derive(Clone)]
pub struct ClaudeClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Content,
}

/// Message content — either a simple string or array of content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// Individual content block within a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: JsonValue,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

/// Tool definition for Claude
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: JsonValue,
}

/// Request body for the Messages API
#[derive(Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

/// Response from the Messages API
#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    #[allow(dead_code)]
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: String,
}

/// Error detail from the Messages API
#[derive(Debug, Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

impl ClaudeClient {
    /// Create a new client with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Send a simple message with an optional system prompt, return text response
    pub async fn message(
        &self,
        system: Option<&str>,
        user_message: &str,
    ) -> Result<String, String> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: Content::Text(user_message.to_string()),
        }];

        let response = self.send(system, messages, None).await?;
        self.extract_text(&response)
    }

    /// Send a full request with messages and optional tools
    pub async fn send(
        &self,
        system: Option<&str>,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ApiResponse, String> {
        let request = ApiRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            system: system.map(|s| s.to_string()),
            messages,
            tools,
        };

        let response = self
            .http
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if let Ok(api_err) = serde_json::from_str::<ApiError>(&body) {
                return Err(format!(
                    "Claude API error ({}): {}",
                    status, api_err.error.message
                ));
            }
            return Err(format!("Claude API error ({}): {}", status, body));
        }

        response
            .json::<ApiResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Extract text content from an API response
    pub fn extract_text(&self, response: &ApiResponse) -> Result<String, String> {
        for block in &response.content {
            if let ContentBlock::Text { text } = block {
                return Ok(text.clone());
            }
        }
        Err("No text content in response".to_string())
    }
}

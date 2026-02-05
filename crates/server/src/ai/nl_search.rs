//! Natural language to FHIR search parameter conversion

use super::client::ClaudeClient;
use serde_json::Value as JsonValue;

const SYSTEM_PROMPT: &str = r#"You are a FHIR search parameter converter. Convert natural language queries about patients into FHIR R4 search parameters.

Return ONLY a JSON object with these possible keys:
- "name": string (patient name to search for)
- "gender": string (must be one of: "male", "female", "other", "unknown")
- "birthdate": string (FHIR date with optional prefix: eq, ne, gt, lt, ge, le, e.g. "ge1990-01-01")

Only include parameters that are relevant to the query. Do not include parameters that weren't mentioned.

Examples:
- "Find all male patients" → {"gender": "male"}
- "Patients named Smith born after 1990" → {"name": "Smith", "birthdate": "ge1990-01-01"}
- "Female patients born before 2000" → {"gender": "female", "birthdate": "lt2000-01-01"}

Return ONLY the JSON object, no other text."#;

/// Convert a natural language query into FHIR search parameters
pub async fn convert_to_params(client: &ClaudeClient, query: &str) -> Result<JsonValue, String> {
    let response = client.message(Some(SYSTEM_PROMPT), query).await?;

    // Parse the JSON from Claude's response (may be wrapped in markdown)
    let json_str = extract_json(&response)?;

    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse search params: {}", e))
}

/// Extract a JSON object from text that might contain markdown code blocks
fn extract_json(text: &str) -> Result<String, String> {
    let trimmed = text.trim();

    // Direct JSON object
    if trimmed.starts_with('{') {
        return Ok(trimmed.to_string());
    }

    // Wrapped in ```json ... ```
    if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        if let Some(end) = after.find("```") {
            return Ok(after[..end].trim().to_string());
        }
    }

    // Wrapped in ``` ... ```
    if let Some(start) = trimmed.find("```") {
        let after = &trimmed[start + 3..];
        if let Some(end) = after.find("```") {
            return Ok(after[..end].trim().to_string());
        }
    }

    Err(format!("Could not extract JSON from response: {}", trimmed))
}

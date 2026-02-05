//! Synthetic FHIR Patient data generation using Claude

use super::client::ClaudeClient;
use serde_json::Value as JsonValue;

const SYSTEM_PROMPT: &str = r#"You are a FHIR R4 Patient resource generator. Generate realistic, diverse patient data.

Each patient MUST be a valid FHIR R4 Patient resource with this structure:
{
  "resourceType": "Patient",
  "name": [{"family": "LastName", "given": ["FirstName"]}],
  "gender": "male|female|other|unknown",
  "birthDate": "YYYY-MM-DD"
}

Requirements:
- Use diverse, realistic names from various cultures
- Mix genders approximately equally
- Use realistic birth dates (between 1930 and 2020)
- Each patient should be unique

Return ONLY a JSON array of Patient resources, no other text."#;

/// Generate synthetic Patient resources using Claude
pub async fn generate_patients(
    client: &ClaudeClient,
    count: u32,
) -> Result<Vec<JsonValue>, String> {
    let user_message = format!(
        "Generate exactly {} unique FHIR R4 Patient resources.",
        count
    );

    let response = client.message(Some(SYSTEM_PROMPT), &user_message).await?;

    // Parse the JSON array from Claude's response
    let json_str = extract_json_array(&response)?;

    let patients: Vec<JsonValue> =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse patients: {}", e))?;

    if patients.len() != count as usize {
        tracing::warn!(
            requested = count,
            generated = patients.len(),
            "Generated patient count mismatch"
        );
    }

    Ok(patients)
}

/// Extract a JSON array from text that might contain markdown code blocks
fn extract_json_array(text: &str) -> Result<String, String> {
    let trimmed = text.trim();

    // Direct JSON array
    if trimmed.starts_with('[') {
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

    Err(format!(
        "Could not extract JSON array from response: {}",
        trimmed
    ))
}

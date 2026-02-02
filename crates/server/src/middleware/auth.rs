use axum::{
    Json,
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use fhir_core::OperationOutcome;

/// API Key authentication state
#[derive(Clone)]
pub struct ApiKeyAuth {
    api_key: Option<String>,
}

impl ApiKeyAuth {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    /// Check if authentication is required and valid
    pub fn validate(&self, headers: &HeaderMap) -> Result<(), Response> {
        // If no API key configured, auth is disabled
        let Some(ref expected_key) = self.api_key else {
            return Ok(());
        };

        // Get the X-API-Key header
        let provided_key = headers.get("X-API-Key").and_then(|v| v.to_str().ok());

        match provided_key {
            Some(key) if key == expected_key => Ok(()),
            Some(_) => {
                let outcome =
                    OperationOutcome::error(fhir_core::IssueType::Security, "Invalid API key");
                Err((StatusCode::UNAUTHORIZED, Json(outcome)).into_response())
            }
            None => {
                let outcome = OperationOutcome::error(
                    fhir_core::IssueType::Security,
                    "Missing X-API-Key header",
                );
                Err((StatusCode::UNAUTHORIZED, Json(outcome)).into_response())
            }
        }
    }
}

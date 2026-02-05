//! API Key authentication middleware

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
    pub fn validate(&self, headers: &HeaderMap) -> Result<(), Box<Response>> {
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
                Err(Box::new(
                    (StatusCode::UNAUTHORIZED, Json(outcome)).into_response(),
                ))
            }
            None => {
                let outcome = OperationOutcome::error(
                    fhir_core::IssueType::Security,
                    "Missing X-API-Key header",
                );
                Err(Box::new(
                    (StatusCode::UNAUTHORIZED, Json(outcome)).into_response(),
                ))
            }
        }
    }
}

/// Middleware function for API key authentication
pub async fn auth_middleware(headers: HeaderMap, request: Request<Body>, next: Next) -> Response {
    // Get auth state from request extensions
    let auth = request
        .extensions()
        .get::<ApiKeyAuth>()
        .cloned()
        .unwrap_or_else(|| ApiKeyAuth::new(None));

    // Validate API key
    if let Err(response) = auth.validate(&headers) {
        return *response;
    }

    next.run(request).await
}

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

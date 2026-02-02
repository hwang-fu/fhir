
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fhir_core::OperationOutcome;

/// Application error type
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

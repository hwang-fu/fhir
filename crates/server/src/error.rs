//! Application error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use fhir_core::OperationOutcome;

/// Application error type
#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, outcome) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, OperationOutcome::not_found(&msg)),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, OperationOutcome::invalid(&msg)),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, OperationOutcome::conflict(&msg)),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                OperationOutcome::error(fhir_core::IssueType::Exception, &msg),
            ),
        };

        (status, Json(outcome)).into_response()
    }
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        AppError::Internal(format!("Database pool error: {}", err))
    }
}

impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        AppError::Internal(format!("Database error: {}", err))
    }
}

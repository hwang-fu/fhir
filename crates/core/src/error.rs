use crate::outcome::{IssueType, OperationOutcome};
use thiserror::Error;

/// FHIR server error types
#[derive(Debug, Error)]
pub enum FhirError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid resource: {0}")]
    Invalid(String),

    #[error("Version conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

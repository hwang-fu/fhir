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

impl FhirError {
    /// Convert to OperationOutcome for FHIR-compliant error responses
    pub fn to_outcome(&self) -> OperationOutcome {
        match self {
            FhirError::NotFound(msg) => OperationOutcome::not_found(msg),
            FhirError::Invalid(msg) => OperationOutcome::invalid(msg),
            FhirError::Conflict(msg) => OperationOutcome::conflict(msg),
            FhirError::Database(msg) => OperationOutcome::error(IssueType::Exception, msg),
            FhirError::Internal(msg) => OperationOutcome::error(IssueType::Exception, msg),
        }
    }
}

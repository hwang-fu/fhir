//! FHIR OperationOutcome for error responses

use serde::{Deserialize, Serialize};

/// Severity of the issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Fatal,
    Error,
    Warning,
    Information,
}

/// Type of issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum IssueType {
    Invalid,
    Structure,
    Required,
    Value,
    Invariant,
    Security,
    Login,
    Unknown,
    Expired,
    Forbidden,
    Suppressed,
    Processing,
    NotSupported,
    Duplicate,
    NotFound,
    TooLong,
    CodeInvalid,
    Extension,
    TooCostly,
    BusinessRule,
    Conflict,
    Incomplete,
    Transient,
    LockError,
    NoStore,
    Exception,
    Timeout,
    Throttled,
    Informational,
}

/// FHIR OperationOutcome resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationOutcome {
    pub resource_type: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issue: Vec<OperationOutcomeIssue>,
}

impl OperationOutcome {
    /// Create a new OperationOutcome with a single issue
    pub fn error(code: IssueType, diagnostics: &str) -> Self {
        Self {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: IssueSeverity::Error,
                code,
                diagnostics: Some(diagnostics.to_string()),
                location: Vec::new(),
            }],
        }
    }

    /// Create a not found error
    pub fn not_found(message: &str) -> Self {
        Self::error(IssueType::NotFound, message)
    }

    /// Create a validation error
    pub fn invalid(message: &str) -> Self {
        Self::error(IssueType::Invalid, message)
    }

    /// Create a conflict error (e.g., version mismatch)
    pub fn conflict(message: &str) -> Self {
        Self::error(IssueType::Conflict, message)
    }

    /// Create a successful validation outcome
    pub fn success(message: &str) -> Self {
        Self {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: IssueSeverity::Information,
                code: IssueType::Informational,
                diagnostics: Some(message.to_string()),
                location: Vec::new(),
            }],
        }
    }
}

/// Individual issue in an OperationOutcome
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationOutcomeIssue {
    pub severity: IssueSeverity,
    pub code: IssueType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub location: Vec<String>,
}

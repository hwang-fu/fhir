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

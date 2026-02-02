//! fhir-core: Shared FHIR R4 types and utilities
//!
//! This crate provides common types used across the FHIR server,
//! including Patient, Bundle, OperationOutcome, and CapabilityStatement.

pub mod bundle;
pub mod error;
pub mod outcome;

// Re-export fhir-sdk types
pub use fhir_sdk::r4b::resources::Patient;
pub use fhir_sdk::r4b::types::{HumanName, Identifier};

// Re-export our types
pub use bundle::{Bundle, BundleEntry, BundleLink, BundleType};
pub use error::FhirError;
pub use outcome::{IssueSeverity, IssueType, OperationOutcome, OperationOutcomeIssue};

//! Metadata endpoint handler

use axum::Json;
use fhir_core::CapabilityStatement;

/// GET /metadata - Return server capability statement
pub async fn get() -> Json<CapabilityStatement> {
    Json(CapabilityStatement::new())
}

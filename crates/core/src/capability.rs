use serde::{Deserialize, Serialize};

/// FHIR CapabilityStatement resource (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityStatement {
    pub resource_type: String,
    pub status: String,
    pub date: String,
    pub kind: String,
    pub fhir_version: String,
    pub format: Vec<String>,
    pub rest: Vec<CapabilityRest>,
}

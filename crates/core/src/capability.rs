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

impl CapabilityStatement {
    /// Create a default capability statement for this server
    pub fn new() -> Self {
        Self {
            resource_type: "CapabilityStatement".to_string(),
            status: "active".to_string(),
            date: "2026-02-02".to_string(),
            kind: "instance".to_string(),
            fhir_version: "4.3.0".to_string(), // R4B
            format: vec!["json".to_string()],
            rest: vec![CapabilityRest::default()],
        }
    }
}

impl Default for CapabilityStatement {
    fn default() -> Self {
        Self::new()
    }
}

/// REST capability declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRest {
    pub mode: String,
    pub resource: Vec<CapabilityResource>,
}

impl Default for CapabilityRest {
    fn default() -> Self {
        Self {
            mode: "server".to_string(),
            resource: vec![CapabilityResource::patient()],
        }
    }
}

/// Resource-level capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub interaction: Vec<CapabilityInteraction>,
    pub versioning: String,
    pub read_history: bool,
    pub search_param: Vec<CapabilitySearchParam>,
}

impl CapabilityResource {
    /// Create Patient resource capabilities
    pub fn patient() -> Self {
        Self {
            resource_type: "Patient".to_string(),
            interaction: vec![
                CapabilityInteraction::new("read"),
                CapabilityInteraction::new("vread"),
                CapabilityInteraction::new("update"),
                CapabilityInteraction::new("delete"),
                CapabilityInteraction::new("history-instance"),
                CapabilityInteraction::new("create"),
                CapabilityInteraction::new("search-type"),
            ],
            versioning: "versioned".to_string(),
            read_history: true,
            search_param: vec![
                CapabilitySearchParam::new("name", "string"),
                CapabilitySearchParam::new("gender", "token"),
                CapabilitySearchParam::new("birthdate", "date"),
            ],
        }
    }
}

/// Supported interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInteraction {
    pub code: String,
}

impl CapabilityInteraction {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
        }
    }
}

/// Search parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySearchParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

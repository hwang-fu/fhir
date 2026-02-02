use serde::{Deserialize, Serialize};

/// FHIR Bundle types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BundleType {
    Searchset,
    History,
    Collection,
    Document,
    Message,
    Transaction,
    TransactionResponse,
    Batch,
    BatchResponse,
}

/// FHIR Bundle resource (simplified for search responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub resource_type: String,

    #[serde(rename = "type")]
    pub bundle_type: BundleType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link: Vec<BundleLink>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<BundleEntry>,
}

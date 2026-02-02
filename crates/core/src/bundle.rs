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

impl Bundle {
    /// Create a new search result bundle
    pub fn searchset(total: u32, entries: Vec<BundleEntry>) -> Self {
        Self {
            resource_type: "Bundle".to_string(),
            bundle_type: BundleType::Searchset,
            total: Some(total),
            link: Vec::new(),
            entry: entries,
        }
    }

    /// Create a new history bundle
    pub fn history(entries: Vec<BundleEntry>) -> Self {
        Self {
            resource_type: "Bundle".to_string(),
            bundle_type: BundleType::History,
            total: Some(entries.len() as u32),
            link: Vec::new(),
            entry: entries,
        }
    }

    /// Add a pagination link
    pub fn add_link(&mut self, relation: &str, url: &str) {
        self.link.push(BundleLink {
            relation: relation.to_string(),
            url: url.to_string(),
        });
    }
}

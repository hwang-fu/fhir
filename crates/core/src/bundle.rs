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

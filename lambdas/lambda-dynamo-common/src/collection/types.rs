use serde::{Deserialize, Serialize};

/// Data of a collection.
#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionData {
    pub block_number: u64,
    pub contract_type: String,
    pub contract_address: String,
}

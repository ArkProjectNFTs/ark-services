use serde::{Deserialize, Serialize};

/// Data of a event.
#[derive(Serialize, Deserialize, Debug)]
pub struct EventData {
    pub block_number: u64,
    pub event_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub from_address: String,
    pub to_address: String,
    pub contract_address: String,
    pub contract_type: String,
    pub token_id: String,
    pub transaction_hash: String,
}

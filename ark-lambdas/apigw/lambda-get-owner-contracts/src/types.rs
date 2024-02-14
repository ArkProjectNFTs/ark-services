use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FullContractData {
    pub contract_address: String,
    pub contract_type: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub image: Option<String>,
    pub tokens_count: u64,
}

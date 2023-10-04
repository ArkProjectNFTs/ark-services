use serde::{Deserialize, Serialize};

/// Data of a token.
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    pub block_number: u64,
    pub mint_timestamp: u64,
    pub mint_address: String,
    pub owner: String,
    pub token_id: String,
    pub contract_address: String,
}

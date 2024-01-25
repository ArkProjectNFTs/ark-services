use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub token_chain_id: String,
    pub token_address: String,
    pub token_id: String,
    pub listed_timestamp: i64,
    pub updated_timestamp: i64,
    pub current_owner: String,
    pub current_price: Option<String>,
    pub quantity: Option<String>,
    pub start_amount: Option<String>,
    pub end_amount: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub broker_id: Option<String>,
    pub is_listed: Option<bool>,
    pub has_offer: Option<bool>
}

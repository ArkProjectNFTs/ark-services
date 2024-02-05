use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub order_hash: String,
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
    pub has_offer: Option<bool>,
    pub currency_address: Option<String>,
    pub currency_chain_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenWithHistory {
    pub token_address: String,
    pub token_id: String,
    pub current_owner: String,
    pub current_price: Option<String>,
    pub history: Vec<TokenHistory>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenHistory {
    pub event_type: String,
    pub event_timestamp: i64,
    pub order_status: String,
    pub previous_owner: Option<String>,
    pub new_owner: Option<String>,
    pub amount: Option<String>,
    pub canceled_reason: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenOffer {
    pub order_hash: String,
    pub offer_maker: String,
    pub offer_amount: String,
    pub offer_quantity: String,
    pub offer_timestamp: i64,
    pub currency_address: Option<String>,
    pub currency_chain_id: Option<String>,
    pub start_date: i64,
    pub end_date: i64,
    pub status: String,


}

#[derive(Serialize, Deserialize)]
pub struct TokenWithOffers {
    pub token_address: String,
    pub token_id: String,
    pub current_owner: String,
    pub current_price: Option<String>,
    pub offers: Vec<TokenOffer>,
}

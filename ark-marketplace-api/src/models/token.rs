use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct TokenData {
    pub contract: String,
    pub token_id: String,
    pub last_price: Option<String>,
    pub floor_difference: i32,
    pub listed_at: Option<String>, // Changed from chrono::NaiveDateTime
    pub price: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub raw_price: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenOneData {
    pub price: Option<BigDecimal>,
    pub last_price: Option<BigDecimal>,
    pub top_offer: Option<BigDecimal>,
    pub owner: Option<String>,
    pub collection_name: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenPortfolioData {
    pub contract: Option<String>,
    pub token_id: Option<String>,
    pub list_price: Option<BigDecimal>,
    pub best_offer: Option<i64>,
    pub floor: Option<BigDecimal>,
    pub received_at: Option<i64>,
    pub metadata: Option<JsonValue>,
    pub collection_name: Option<String>,
}

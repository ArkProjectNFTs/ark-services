use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenData {
    pub contract: Option<String>,
    pub token_id: Option<String>,
    pub last_price: Option<BigDecimal>,
    pub floor_difference: Option<BigDecimal>,
    pub listed_at: Option<i64>,
    pub owner: Option<String>,
    pub minted_at: i64,
    pub updated_at: i64,
    pub price: Option<BigDecimal>,
    pub metadata: Option<JsonValue>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenPortfolioData {
    pub contract: Option<String>,
    pub token_id: Option<String>,
    pub list_price: Option<BigDecimal>,
    pub best_offer: Option<BigDecimal>,
    pub floor: Option<BigDecimal>,
    pub received_at: Option<i64>,
    pub metadata: Option<JsonValue>,
}

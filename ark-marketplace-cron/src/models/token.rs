use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenData {
    pub contract: Option<String>,
    pub token_id: Option<String>,
    pub last_price: Option<BigDecimal>,
    pub floor_difference: Option<i32>,
    pub listed_at: Option<i64>,
    pub price: Option<BigDecimal>,
    pub metadata: Option<JsonValue>,
}

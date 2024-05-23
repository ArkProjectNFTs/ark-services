use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenData {
    pub contract: String,
    pub token_id: Option<String>,
    pub floor: Option<String>,
    pub floor_7d_percentage: Option<i32>,
    pub volume_7d_eth: Option<i32>,
    pub top_offer: Option<String>,
    pub sales_7d: Option<i64>,
    pub marketcap: Option<i32>,
    pub listed_items: Option<i64>,
    pub listed_percentage: Option<i64>,
}

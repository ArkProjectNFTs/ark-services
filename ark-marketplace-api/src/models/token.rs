use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenData {
    pub contract: Option<String>,
    pub token_id: Option<String>,
    pub owner: Option<String>,
    pub minted_at: i64,
    pub updated_at: i64
}


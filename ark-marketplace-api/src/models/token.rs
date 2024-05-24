use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenData {
    pub contract: String,
    pub token_id: Option<String>,
    pub isNsfw: bool,
    pub isSpam: bool,
    pub collection: Collection,
    pub owner: String,
    pub mintedAt: i64,
    pub updatedAt: i64
}


#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub image: String,
    pub symbol: String,
    pub tokenCount: i32,
}

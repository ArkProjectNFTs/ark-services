use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::models::token::TokenEventType;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct CollectionData {
    pub address: String,
    pub image: Option<String>,
    pub name: Option<String>,
    pub floor: Option<BigDecimal>,
    pub volume_7d_eth: Option<BigDecimal>,
    pub top_offer: Option<BigDecimal>,
    pub sales_7d: Option<i64>,
    pub marketcap: Option<BigDecimal>,
    pub listed_items: Option<i64>,
    pub listed_percentage: Option<i64>,
    pub token_count: Option<i64>,
    pub owner_count: Option<i64>,
    pub total_volume: Option<i64>,
    pub total_sales: Option<i64>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct CollectionPortfolioData {
    pub address: String,
    pub image: Option<String>,
    pub name: Option<String>,
    pub floor: Option<BigDecimal>,
    pub token_count: Option<i64>,
    pub user_token_count: Option<i64>,
    pub user_listed_tokens: Option<i64>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct CollectionSearchData {
    pub name: Option<String>,
    pub address: String,
    pub image: Option<String>,
    pub token_count: Option<i64>,
    pub is_verified: Option<bool>,
}

#[derive(FromRow)]
pub struct CollectionFloorPrice {
    pub value: Option<BigDecimal>,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct CollectionActivityData {
    pub activity_type: TokenEventType,
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
    pub transaction_hash: Option<String>,
    pub token_id: Option<String>,
}

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct TokenData {
    pub contract: Option<String>,
    pub token_id: Option<String>,
    pub last_price: Option<BigDecimal>,
    pub floor_difference: Option<i32>,
    pub listed_at: Option<i64>,
    pub price: Option<BigDecimal>,
    pub metadata: Option<JsonValue>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenMarketData {
    pub owner: Option<String>,
    pub floor: Option<BigDecimal>,
    pub created_timestamp: Option<i64>,
    pub updated_timestamp: Option<i64>,
    pub is_listed: Option<bool>,
    pub has_offer: Option<bool>,
    pub buy_in_progress: Option<bool>,
    pub top_offer: Option<TopOffer>,
    pub listing: Option<Listing>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenInformationData {
    pub price: Option<BigDecimal>,
    pub last_price: Option<BigDecimal>,
    pub top_offer: Option<BigDecimal>,
    pub owner: Option<String>,
    pub collection_name: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TokenOneData {
    pub owner: Option<String>,
    pub floor: Option<BigDecimal>,
    pub created_timestamp: Option<i64>,
    pub updated_timestamp: Option<i64>,
    pub is_listed: Option<bool>,
    pub has_offer: Option<bool>,
    pub buy_in_progress: Option<bool>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TopOffer {
    pub order_hash: Option<String>,
    pub amount: Option<BigDecimal>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub currency_address: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Listing {
    pub is_auction: Option<bool>,
    pub order_hash: Option<String>,
    pub start_amount: Option<String>,
    pub end_amount: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub currency_address: Option<String>,
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
    pub collection_name: Option<String>,
}

#[derive(FromRow)]
pub struct TokenOfferOneDataDB {
    pub offer_id: i32,
    pub amount: Option<BigDecimal>,
    pub currency_address: String,
    pub source: Option<String>,
    pub expire_at: i64,
    pub hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenOfferOneData {
    pub offer_id: i32,
    pub price: Option<BigDecimal>,
    pub floor_difference: Option<BigDecimal>,
    pub source: Option<String>,
    pub expire_at: i64,
    pub hash: Option<String>,
}

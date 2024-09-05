use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OfferData {
    pub offer_id: i32,
    pub amount: Option<BigDecimal>,
    pub currency_address: String,
    pub source: Option<String>,
    pub expire_at: i64,
    pub hash: Option<String>,
    pub token_id: Option<String>,
    pub to_address: Option<String>,
    pub collection_floor_price: Option<BigDecimal>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OfferApiData {
    pub offer_id: i32,
    pub price: Option<BigDecimal>,
    pub currency_address: String,
    pub expire_at: i64,
    pub hash: Option<String>,
    pub token_id: Option<String>,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    pub floor_difference: Option<BigDecimal>,
}

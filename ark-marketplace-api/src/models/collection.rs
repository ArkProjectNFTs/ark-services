use super::default::Currency;
use crate::models::token::TokenEventType;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone, utoipa::ToSchema)]
pub struct CollectionFullData {
    #[schema(value_type = String, example = "0x02acee8c430f62333cf0e0e7a94b2347b5513b4c25f699461dd8d7b23c072478")]
    pub address: String,
    pub image: Option<String>,
    pub name: Option<String>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub floor: Option<BigDecimal>,
    pub volume: Option<i64>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub top_offer: Option<BigDecimal>,
    pub sales: Option<i64>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub marketcap: Option<BigDecimal>,
    pub listed_items: Option<i64>,
    pub listed_percentage: Option<i64>,
    pub token_count: Option<i64>,
    pub owner_count: Option<i64>,
    pub total_volume: Option<i64>,
    pub total_sales: Option<i64>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub floor_percentage: Option<i64>,
    pub is_verified: Option<bool>,
}

#[derive(Serialize, Deserialize, FromRow, Clone, utoipa::ToSchema)]
pub struct CollectionData {
    #[schema(value_type = String, example = "0x02acee8c430f62333cf0e0e7a94b2347b5513b4c25f699461dd8d7b23c072478")]
    pub address: String,
    pub image: Option<String>,
    pub name: Option<String>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub floor: Option<BigDecimal>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub volume_7d_eth: Option<BigDecimal>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub top_offer: Option<BigDecimal>,
    pub sales_7d: Option<i64>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub marketcap: Option<BigDecimal>,
    pub listed_items: Option<i64>,
    pub listed_percentage: Option<i64>,
    pub token_count: Option<i64>,
    pub owner_count: Option<i64>,
    pub total_volume: Option<BigDecimal>,
    pub total_sales: Option<i64>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub floor_7d_percentage: Option<BigDecimal>,
    pub is_verified: Option<bool>,
    pub deployed_timestamp: Option<i64>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub twitter: Option<String>,
    pub discord: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct CollectionPortfolioData {
    pub address: String,
    pub image: Option<String>,
    pub name: Option<String>,
    #[schema(value_type = String, example = "1000000000000000")]
    pub floor: Option<BigDecimal>,
    #[schema(value_type = String, example = "777")]
    pub token_count: Option<i64>,
    #[schema(value_type = String, example = "12")]
    pub user_token_count: Option<i64>,
    #[schema(value_type = String, example = "2")]
    pub user_listed_tokens: Option<i64>,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema)]
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

#[derive(Deserialize, Serialize, FromRow)]
pub struct CollectionActivityDataDB {
    pub activity_type: TokenEventType,
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
    pub transaction_hash: Option<String>,
    pub token_id: Option<String>,
    pub token_metadata: Option<JsonValue>,
    pub name: Option<String>,
    pub address: String,
    pub is_verified: Option<bool>,
    pub currency_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, utoipa::ToSchema)]
pub struct CollectionActivityData {
    pub activity_type: TokenEventType,
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
    pub transaction_hash: Option<String>,
    pub token_id: Option<String>,
    #[schema(
        value_type = Object,
        example = r#"{
            "name": "Starknet ID: 154773638476",
            "image": "https://starknet.id/api/identicons/154773638476",
            "description": "This token represents an identity on StarkNet.",
            "image_mime_type": "image/svg+xml",
            "external_url": null,
            "properties": null
        }"#
    )]
    pub token_metadata: Option<JsonValue>,
    pub name: Option<String>,
    pub address: String,
    pub is_verified: Option<bool>,
    #[schema(
        example = r#"{"contract": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7", "symbol": "ETH", "decimals": 18}"#
    )]
    pub currency: Option<Currency>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, utoipa::ToSchema)]
pub struct OwnerData {
    pub owner: String,
    pub chain_id: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct OwnerDataCompleted {
    pub owner: String,
    pub chain_id: String,
    pub starknet_id: Option<String>,
    pub image: Option<String>,
}

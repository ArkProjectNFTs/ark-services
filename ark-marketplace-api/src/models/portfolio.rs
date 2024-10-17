use super::default::Currency;
use crate::models::token::Listing;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use sqlx::Row;

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
    pub collection_address: String,
    pub collection_name: Option<String>,
    pub is_verified: bool,
    pub metadata: Option<JsonValue>,
    pub is_listed: bool,
    pub listing: Option<Listing>,
}

impl<'r> FromRow<'r, sqlx::postgres::PgRow> for OfferData {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(OfferData {
            offer_id: row.try_get("offer_id")?,
            amount: row.try_get("amount")?,
            currency_address: row.try_get("currency_address")?,
            source: row.try_get("source")?,
            expire_at: row.try_get("expire_at")?,
            hash: row.try_get("hash")?,
            token_id: row.try_get("token_id")?,
            to_address: row.try_get("to_address")?,
            collection_floor_price: row.try_get("collection_floor_price")?,
            collection_address: row.try_get("collection_address")?,
            collection_name: row.try_get("collection_name")?,
            is_verified: row.try_get("is_verified")?,
            metadata: row.try_get("metadata")?,
            is_listed: row.try_get("is_listed")?,
            listing: Some(Listing {
                is_auction: row.try_get("is_auction")?,
                order_hash: row.try_get("listing_order_hash")?,
                start_amount: row.try_get("listing_start_amount")?,
                end_amount: row.try_get("listing_end_amount")?,
                start_date: row.try_get("listing_start_date")?,
                end_date: row.try_get("listing_end_date")?,
                currency: Currency {
                    contract: row.try_get("currency_address")?,
                    symbol: row.try_get("currency_symbol")?,
                    decimals: row.try_get("currency_decimals")?,
                },
            }),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct OfferApiData {
    pub offer_id: i32,
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
    pub currency_address: String,
    pub expire_at: i64,
    pub hash: Option<String>,
    pub token_id: Option<String>,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    #[schema(value_type = String, example = "12345.6789")]
    pub floor_difference: Option<BigDecimal>,
    pub collection_address: String,
    pub collection_name: Option<String>,
    pub listing: Option<Listing>,
    pub is_listed: bool,
    pub is_verified: bool,
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
    pub metadata: Option<JsonValue>,
}

#[derive(Serialize, Deserialize, Clone, FromRow, utoipa::ToSchema)]
pub struct StatsData {
    #[schema(value_type = String, example = "12345.6789")]
    pub total_value: Option<BigDecimal>,
}

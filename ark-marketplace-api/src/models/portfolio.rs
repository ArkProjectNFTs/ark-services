use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, FromRow)]
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

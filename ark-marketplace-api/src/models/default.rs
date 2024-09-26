use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, FromRow, utoipa::ToSchema)]
pub struct LastSale {
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
    pub collection_name: String,
    pub collection_address: String,
    pub price: Option<BigDecimal>,
    pub from: String,
    pub to: String,
    pub timestamp: Option<i64>,
    pub transaction_hash: Option<String>,
}

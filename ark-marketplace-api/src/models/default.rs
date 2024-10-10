use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use sqlx::Row;

#[derive(Serialize, Deserialize, Debug, FromRow, utoipa::ToSchema, Clone)]
pub struct Currency {
    pub contract: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<i16>,
}

#[derive(FromRow)]
pub struct LastSaleDB {
    pub metadata: Option<JsonValue>,
    pub collection_name: String,
    pub collection_address: String,
    pub price: Option<BigDecimal>,
    pub from: String,
    pub to: String,
    pub timestamp: Option<i64>,
    pub transaction_hash: Option<String>,
    pub token_id: Option<String>,
    pub currency_address: Option<String>,
}

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
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
    pub from: String,
    pub to: String,
    pub timestamp: Option<i64>,
    pub transaction_hash: Option<String>,
    pub token_id: Option<String>,
    #[schema(
        example = r#"{"contract": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7", "symbol": "ETH", "decimals": 18}"#
    )]
    pub currency: Option<Currency>,
}

#[derive(Serialize, Deserialize, Clone, FromRow, utoipa::ToSchema)]
pub struct LiveAuction {
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
    pub end_timestamp: Option<i64>,
    pub collection_address: Option<String>,
    pub token_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRow, utoipa::ToSchema)]
pub struct PreviewNft {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CollectionInfo {
    pub collection_name: String,
    pub collection_address: String,
    pub collection_image: String,
    #[schema(value_type = String, example = "12345.6789")]
    pub floor_price: Option<BigDecimal>,
    pub floor_difference: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Trending {
    pub preview_nfts: Vec<PreviewNft>,
    pub collection_name: String,
    pub collection_address: String,
    pub collection_image: String,
    #[schema(value_type = String, example = "12345.6789")]
    pub floor_price: Option<BigDecimal>,
    pub floor_difference: Option<i64>,
}

impl<'r> FromRow<'r, sqlx::postgres::PgRow> for CollectionInfo {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let floor_price: Option<BigDecimal> = row.try_get("floor_price")?;
        let floor_difference: Option<i64> = row.try_get("floor_difference")?;
        let collection_address: String = row.try_get("collection_address")?;
        let collection_name: String = row.try_get("collection_name")?;
        let collection_image: String = row.try_get("collection_image")?;

        Ok(CollectionInfo {
            collection_address,
            collection_name,
            collection_image,
            floor_price,
            floor_difference,
        })
    }
}

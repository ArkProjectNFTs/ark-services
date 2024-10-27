use std::str::FromStr;

use super::default::Currency;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

/// DEV-690: Expected format for Token Data
/// {
///  "data": [
///    {
///      "collection_address": "0x032d99485b22f2e58c8a0206d3b3bb259997ff0db70cffd25585d7dd9a5b0546",
///      "collection_name": "ARKTEST",
///      "floor_difference": 0,
///      "last_price": "1700000000000000",
///      "listed_at": 1721332538,
///      "is_listed": true,
///      "listing": {
///        is_auction: false,
///      },
#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TokenDataListing {
    pub is_auction: Option<bool>,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TokenData {
    pub collection_address: Option<String>,
    pub token_id: Option<String>,
    #[schema(value_type = String, example = "12345.6789")]
    pub last_price: Option<BigDecimal>,
    pub floor_difference: Option<i32>,
    pub listed_at: Option<i64>,
    pub is_listed: Option<bool>,
    pub listing: Option<TokenDataListing>,
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
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
    pub owner: Option<String>,
    pub buy_in_progress: Option<bool>,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TokenMarketData {
    pub owner: Option<String>,
    #[schema(value_type = String, example = "12345.6789")]
    pub floor: Option<BigDecimal>,
    pub created_timestamp: Option<i64>,
    pub updated_timestamp: Option<i64>,
    pub is_listed: Option<bool>,
    pub has_offer: Option<bool>,
    pub buy_in_progress: Option<bool>,
    pub top_offer: Option<TopOffer>,
    pub listing: Option<Listing>,
    #[schema(value_type = String, example = "12345.6789")]
    pub last_price: Option<BigDecimal>,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TokenInformationData {
    pub token_id: String,
    pub collection_address: String,
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
    #[schema(value_type = String, example = "12345.6789")]
    pub last_price: Option<BigDecimal>,
    #[schema(value_type = String, example = "12345.6789")]
    pub top_offer: Option<BigDecimal>,
    pub owner: Option<String>,
    pub collection_name: Option<String>,
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
    pub collection_image: Option<String>,
    pub metadata_updated_at: Option<i64>,
    pub metadata_status: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TokenOneData {
    pub owner: Option<String>,
    #[schema(value_type = String, example = "12345.6789")]
    pub floor: Option<BigDecimal>,
    pub created_timestamp: Option<i64>,
    pub updated_timestamp: Option<i64>,
    pub is_listed: Option<bool>,
    pub has_offer: Option<bool>,
    pub buy_in_progress: Option<bool>,
    #[schema(value_type = String, example = "12345.6789")]
    pub last_price: Option<BigDecimal>,
    pub listing_currency_address: Option<String>,
    pub listing_currency_chain_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct TopOfferQueryResult {
    pub order_hash: Option<String>,
    pub amount: Option<BigDecimal>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub currency_address: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TopOffer {
    pub order_hash: Option<String>,
    #[schema(value_type = String, example = "12345.6789")]
    pub amount: Option<BigDecimal>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema, Clone)]
pub struct ListingRaw {
    pub is_auction: Option<bool>,
    pub order_hash: Option<String>,
    pub start_amount: Option<String>,
    pub end_amount: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema, Clone)]
pub struct Listing {
    pub is_auction: Option<bool>,
    pub order_hash: Option<String>,
    pub start_amount: Option<String>,
    pub end_amount: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TokenPortfolioData {
    pub collection_address: Option<String>,
    pub token_id: Option<String>,
    #[schema(value_type = String, example = "12345.6789")]
    pub list_price: Option<BigDecimal>,
    #[schema(value_type = String, example = "12345.6789")]
    pub best_offer: Option<BigDecimal>,
    pub floor: Option<BigDecimal>,
    pub received_at: Option<i64>,
    #[schema(example = json!({
        "name": "Starknet ID: 154773638476",
        "image": "https://starknet.id/api/identicons/154773638476",
        "description": "This token represents an identity on StarkNet.",
        "image_mime_type": "image/svg+xml",
        "external_url": null,
        "properties": null
    }))]
    pub metadata: Option<JsonValue>,
    pub collection_name: Option<String>,
    pub currency_address: Option<String>,
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

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TokenOfferOneData {
    pub offer_id: i32,
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
    pub currency: Currency,
    #[schema(value_type = String, example = "12345.6789")]
    pub floor_difference: Option<BigDecimal>,
    pub source: Option<String>,
    pub expire_at: i64,
    pub hash: Option<String>,
}

#[derive(Debug, utoipa::ToSchema)]
pub enum TokenEventType {
    Listing,
    CollectionOffer,
    Offer,
    Auction,
    Fulfill,
    Cancelled,
    Executed,
    Sale,
    Mint,
    Burn,
    Transfer,
    // Cancel event type
    ListingCancelled,
    AuctionCancelled,
    OfferCancelled,
    // Expired event type
    ListingExpired,
    OfferExpired,
}

const LISTING_STR: &str = "LISTING";
const COLLECTION_OFFER_STR: &str = "COLLECTION_OFFER";
const OFFER_STR: &str = "OFFER";
const AUCTION_STR: &str = "AUCTION";
const FULFILL_STR: &str = "FULFILL";
const CANCELLED_STR: &str = "CANCELLED";
const EXECUTED_STR: &str = "EXECUTED";
const SALE_STR: &str = "SALE";
const MINT_STR: &str = "MINT";
const BURN_STR: &str = "BURN";
const TRANSFER_STR: &str = "TRANSFER";
const LISTING_CANCELLED_STR: &str = "DELISTING";
const AUCTION_CANCELLED_STR: &str = "CANCEL_AUCTION";
const OFFER_CANCELLED_STR: &str = "CANCEL_OFFER";
const LISTING_EXPIRED_STR: &str = "EXPIRED_LISTING";
const OFFER_EXPIRED_STR: &str = "EXPIRED_OFFER";

const VARIANTS: [&str; 16] = [
    LISTING_STR,
    COLLECTION_OFFER_STR,
    OFFER_STR,
    AUCTION_STR,
    FULFILL_STR,
    CANCELLED_STR,
    EXECUTED_STR,
    SALE_STR,
    MINT_STR,
    BURN_STR,
    TRANSFER_STR,
    LISTING_CANCELLED_STR,
    AUCTION_CANCELLED_STR,
    OFFER_CANCELLED_STR,
    LISTING_EXPIRED_STR,
    OFFER_EXPIRED_STR,
];

impl std::fmt::Display for TokenEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Listing => write!(f, "{}", LISTING_STR),
            Self::CollectionOffer => write!(f, "{}", COLLECTION_OFFER_STR),
            Self::Offer => write!(f, "{}", OFFER_STR),
            Self::Auction => write!(f, "{}", AUCTION_STR),
            Self::Fulfill => write!(f, "{}", FULFILL_STR),
            Self::Cancelled => write!(f, "{}", CANCELLED_STR),
            Self::Executed => write!(f, "{}", EXECUTED_STR),
            Self::Sale => write!(f, "{}", SALE_STR),
            Self::Mint => write!(f, "{}", MINT_STR),
            Self::Burn => write!(f, "{}", BURN_STR),
            Self::Transfer => write!(f, "{}", TRANSFER_STR),

            Self::ListingCancelled => write!(f, "{}", LISTING_CANCELLED_STR),
            Self::AuctionCancelled => write!(f, "{}", AUCTION_CANCELLED_STR),
            Self::OfferCancelled => write!(f, "{}", OFFER_CANCELLED_STR),

            Self::ListingExpired => write!(f, "{}", LISTING_EXPIRED_STR),
            Self::OfferExpired => write!(f, "{}", OFFER_EXPIRED_STR),
        }
    }
}

impl FromStr for TokenEventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            LISTING_STR => Ok(Self::Listing),
            COLLECTION_OFFER_STR => Ok(Self::CollectionOffer),
            OFFER_STR => Ok(Self::Offer),
            AUCTION_STR => Ok(Self::Auction),
            FULFILL_STR => Ok(Self::Fulfill),
            CANCELLED_STR => Ok(Self::Cancelled),
            EXECUTED_STR => Ok(Self::Executed),
            SALE_STR => Ok(Self::Sale),
            MINT_STR => Ok(Self::Mint),
            BURN_STR => Ok(Self::Burn),
            TRANSFER_STR => Ok(Self::Transfer),
            // cancel event
            LISTING_CANCELLED_STR => Ok(Self::ListingCancelled),
            AUCTION_CANCELLED_STR => Ok(Self::AuctionCancelled),
            OFFER_CANCELLED_STR => Ok(Self::OfferCancelled),
            LISTING_EXPIRED_STR => Ok(Self::ListingExpired),
            OFFER_EXPIRED_STR => Ok(Self::OfferExpired),
            _ => Err(format!("Invalid variant: {} ({})", s, VARIANTS.join(", "))),
        }
    }
}

impl<'de> Deserialize<'de> for TokenEventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TokenEventType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for TokenEventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct TokenMetadataInfo {
    pub metadata_status: String,
    pub metadata_updated_at: i64,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct TokenActivityDataDB {
    pub activity_type: TokenEventType,
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
    pub transaction_hash: Option<String>,
    pub metadata: Option<JsonValue>,
    pub collection_name: Option<String>,
    pub collection_is_verified: Option<bool>,
    pub collection_address: Option<String>,
    pub currency_address: Option<String>,
}

#[derive(Deserialize, Serialize, FromRow, utoipa::ToSchema)]
pub struct TokenActivityData {
    pub activity_type: TokenEventType,
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
    pub transaction_hash: Option<String>,
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
    pub collection_name: Option<String>,
    pub collection_is_verified: Option<bool>,
    pub collection_address: Option<String>,
    pub currency: Currency,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct TokenPortfolioActivityDataDB {
    pub activity_type: TokenEventType,
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
    pub transaction_hash: Option<String>,
    pub token_id: Option<String>,
    pub collection_address: Option<String>,
    pub collection_name: Option<String>,
    pub collection_is_verified: Option<bool>,
    pub metadata: Option<JsonValue>,
    pub currency_address: Option<String>,
}

#[derive(Deserialize, Serialize, FromRow, utoipa::ToSchema)]
pub struct TokenPortfolioActivityData {
    pub activity_type: TokenEventType,
    #[schema(value_type = String, example = "12345.6789")]
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
    pub transaction_hash: Option<String>,
    pub token_id: Option<String>,
    pub collection_address: Option<String>,
    pub collection_name: Option<String>,
    pub collection_is_verified: Option<bool>,
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
    pub currency: Currency,
}

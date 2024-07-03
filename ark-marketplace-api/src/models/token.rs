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
    pub order_hash: String,
    pub amount: String,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub currency_address: String,
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
    pub best_offer: Option<i64>,
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

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "text")]
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
}

const LISTING_STR: &str = "Listing";
const COLLECTION_OFFER_STR: &str = "CollectionOffer";
const OFFER_STR: &str = "Offer";
const AUCTION_STR: &str = "Auction";
const FULFILL_STR: &str = "Fulfill";
const CANCELLED_STR: &str = "Cancelled";
const EXECUTED_STR: &str = "Executed";
const SALE_STR: &str = "Sale";
const MINT_STR: &str = "Mint";
const BURN_STR: &str = "Burn";
const TRANSFER_STR: &str = "Transfer";

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
        }
    }
}

impl std::str::FromStr for TokenEventType {
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
            _ => Err(format!("Invalid variant: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct TokenActivityData {
    pub activity_type: TokenEventType,
    pub price: Option<BigDecimal>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub time_stamp: i64,
}

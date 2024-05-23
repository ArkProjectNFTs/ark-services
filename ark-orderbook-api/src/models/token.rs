use serde::{Deserialize, Serialize};

impl From<RawTokenData> for TokenData {
    fn from(raw_data: RawTokenData) -> Self {
        let top_bid = TopBid {
            amount: raw_data.top_bid_amount,
            order_hash: raw_data.top_bid_order_hash,
        };

        let broker_id = raw_data
            .broker_id
            .as_deref()
            .map(|id| id.parse::<i64>())
            .transpose()
            .unwrap_or(None);

        TokenData {
            order_hash: raw_data.order_hash,
            token_chain_id: raw_data.token_chain_id,
            token_address: raw_data.token_address,
            token_id: raw_data.token_id,
            listed_timestamp: raw_data.listed_timestamp,
            updated_timestamp: raw_data.updated_timestamp,
            current_owner: raw_data.current_owner,
            last_price: raw_data.last_price,
            quantity: raw_data.quantity,
            start_amount: raw_data.start_amount,
            end_amount: raw_data.end_amount,
            start_date: raw_data.start_date,
            end_date: raw_data.end_date,
            broker_id,
            is_listed: raw_data.is_listed,
            has_offer: raw_data.has_offer,
            currency_chain_id: raw_data.currency_chain_id,
            currency_address: raw_data.currency_address,
            top_bid,
            status: raw_data.status,
            buy_in_progress: raw_data.buy_in_progress,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct RawTokenData {
    pub order_hash: String,
    pub token_chain_id: String,
    pub token_id: String,
    pub token_address: String,
    pub listed_timestamp: i64,
    pub updated_timestamp: i64,
    pub current_owner: Option<String>,
    pub last_price: Option<String>,
    pub quantity: Option<String>,
    pub start_amount: Option<String>,
    pub end_amount: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub broker_id: Option<String>,
    pub is_listed: Option<bool>,
    pub has_offer: Option<bool>,
    pub currency_chain_id: Option<String>,
    pub currency_address: Option<String>,
    pub top_bid_amount: Option<String>,
    pub top_bid_order_hash: Option<String>,
    pub status: String,
    pub buy_in_progress: Option<bool>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct TopBid {
    pub amount: Option<String>,
    pub order_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub order_hash: String,
    pub token_chain_id: String,
    pub token_address: String,
    pub token_id: String,
    pub listed_timestamp: i64,
    pub updated_timestamp: i64,
    pub current_owner: Option<String>,
    pub last_price: Option<String>,
    pub quantity: Option<String>,
    pub start_amount: Option<String>,
    pub end_amount: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub broker_id: Option<i64>,
    pub is_listed: Option<bool>,
    pub has_offer: Option<bool>,
    pub currency_address: Option<String>,
    pub currency_chain_id: Option<String>,
    pub top_bid: TopBid,
    pub status: String,
    pub buy_in_progress: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenWithHistory {
    pub token_address: String,
    pub token_id: String,
    pub current_owner: Option<String>,
    pub last_price: Option<String>,
    pub history: Vec<TokenHistory>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenHistory {
    pub event_type: String,
    pub event_timestamp: i64,
    pub order_status: String,
    pub previous_owner: Option<String>,
    pub new_owner: Option<String>,
    pub amount: Option<String>,
    pub canceled_reason: Option<String>,
    pub end_amount: Option<String>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenOffer {
    pub order_hash: String,
    pub offer_maker: String,
    pub offer_amount: String,
    pub offer_quantity: String,
    pub offer_timestamp: i64,
    pub currency_address: Option<String>,
    pub currency_chain_id: Option<String>,
    pub start_date: i64,
    pub end_date: i64,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenWithOffers {
    pub token_address: String,
    pub token_id: String,
    pub current_owner: Option<String>,
    pub last_price: Option<String>,
    pub offers: Vec<TokenOffer>,
}

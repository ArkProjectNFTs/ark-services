use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::interfaces::contract::TransactionInfo;

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct TokenEvent {
    pub token_event_id: String,
    pub contract_address: String,
    pub chain_id: String,
    pub broker_id: Option<String>,
    pub order_hash: Option<String>,
    pub token_id: String,
    #[serde(rename = "event_type")]
    pub event_type: TokenEventType,
    pub block_timestamp: u64,
    pub transaction_hash: Option<String>,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    pub amount: Option<String>,
    pub canceled_reason: Option<String>,
    pub token_sub_event_id: String,
}

#[derive(Debug, sqlx::Type, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum TokenEventType {
    Listing,
    CollectionOffer,
    Offer,
    Auction,
    Cancelled,
    Sale,
    Mint,
    Burn,
    Transfer,
    Rollback,
    ListingCancelled,
    AuctionCancelled,
    OfferCancelled,
    ListingExpired,
    OfferExpired,
}

#[derive(Debug, Error)]
pub enum TokenEventError {
    #[error("Token ID must be defined")]
    MissingTokenId,
}

impl TryFrom<TransactionInfo> for TokenEvent {
    type Error = TokenEventError;

    fn try_from(tx: TransactionInfo) -> Result<Self, Self::Error> {
        let token_id = tx.token_id.ok_or(TokenEventError::MissingTokenId)?;

        Ok(Self {
            token_event_id: format!("{}_{}", tx.tx_hash, tx.event_id),
            contract_address: tx.contract_address,
            chain_id: tx.chain_id,
            token_id: token_id.to_string(),
            event_type: TokenEventType::Transfer,
            block_timestamp: tx.timestamp,
            transaction_hash: Some(tx.tx_hash),
            to_address: Some(tx.to),
            from_address: Some(tx.from),
            amount: tx.value.map(|v| v.to_string()),
            canceled_reason: None,
            broker_id: None,
            order_hash: None,
            token_sub_event_id: tx.sub_event_id,
        })
    }
}

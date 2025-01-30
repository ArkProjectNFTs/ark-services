use crate::interfaces::contract::TransactionInfo;
use crate::interfaces::event::EventType;
use crate::interfaces::orderbook::OrderbookTransactionInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    // Orderbook events
    Listing,
    CollectionOffer,
    Offer,
    Auction,
    Cancelled,
    Sale,
    Rollback,
    ListingCancelled,
    AuctionCancelled,
    OfferCancelled,
    ListingExpired,
    OfferExpired,

    // Transfer events
    Mint,
    Burn,
    Transfer,
}

#[derive(Debug, Error)]
pub enum TokenEventError {
    #[error("Not implemented")]
    NotImplemented,
    #[error("Token ID must be defined")]
    MissingTokenId,
    #[error("Unknown event type")]
    UnknownEventType,
}

impl TryFrom<TransactionInfo> for TokenEvent {
    type Error = TokenEventError;

    fn try_from(tx: TransactionInfo) -> Result<Self, Self::Error> {
        if tx.token_id.is_none() {
            return Err(TokenEventError::MissingTokenId);
        }

        if !matches!(
            tx.event_type,
            EventType::Transfer
                | EventType::TransferSingle
                | EventType::TransferBatch
                | EventType::TransferByPartition
        ) {
            return Err(TokenEventError::UnknownEventType);
        }

        let event_type = if tx.from == "0x0" {
            TokenEventType::Mint
        } else if tx.to == "0x0" {
            TokenEventType::Burn
        } else {
            TokenEventType::Transfer
        };

        Ok(Self {
            token_event_id: format!("{}_{}", tx.tx_hash, tx.event_id),
            contract_address: tx.contract_address,
            chain_id: tx.chain_id,
            token_id: tx.token_id.unwrap().to_string(),
            event_type,
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

impl TryFrom<OrderbookTransactionInfo> for TokenEvent {
    type Error = TokenEventError;

    fn try_from(_tx: OrderbookTransactionInfo) -> Result<Self, Self::Error> {
        Err(TokenEventError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interfaces::contract::ContractType;
    use sqlx::types::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn test_try_from_transaction_info_mint() {
        let tx = TransactionInfo {
            tx_hash: "hash123".to_string(),
            event_id: 1,
            chain_id: "chain1".to_string(),
            from: "0x0".to_string(),
            to: "0xrecipient".to_string(),
            value: Some(BigDecimal::from_str("1000").unwrap()),
            timestamp: 12345,
            token_id: Some(BigDecimal::from_str("1").unwrap()),
            event_type: EventType::Transfer,
            compliance: crate::interfaces::event::ERCCompliance::OPENZEPPELIN,
            action: crate::interfaces::event::ErcAction::MINT,
            contract_address: "0xcontract".to_string(),
            contract_type: ContractType::ERC721,
            block_hash: "blockhash".to_string(),
            sub_event_id: "sub123".to_string(),
        };

        let token_event = TokenEvent::try_from(tx).unwrap();
        assert_eq!(token_event.event_type, TokenEventType::Mint);
        assert_eq!(token_event.token_id, "1");
        assert_eq!(token_event.from_address, Some("0x0".to_string()));
    }

    #[test]
    fn test_try_from_transaction_info_burn() {
        let tx = TransactionInfo {
            tx_hash: "hash123".to_string(),
            event_id: 1,
            chain_id: "chain1".to_string(),
            from: "0xsender".to_string(),
            to: "0x0".to_string(),
            value: Some(BigDecimal::from_str("1000").unwrap()),
            timestamp: 12345,
            token_id: Some(BigDecimal::from_str("1").unwrap()),
            event_type: EventType::Transfer,
            compliance: crate::interfaces::event::ERCCompliance::OPENZEPPELIN,
            action: crate::interfaces::event::ErcAction::BURN,
            contract_address: "0xcontract".to_string(),
            contract_type: ContractType::ERC721,
            block_hash: "blockhash".to_string(),
            sub_event_id: "sub123".to_string(),
        };

        let token_event = TokenEvent::try_from(tx).unwrap();
        assert_eq!(token_event.event_type, TokenEventType::Burn);
        assert_eq!(token_event.to_address, Some("0x0".to_string()));
    }

    #[test]
    fn test_try_from_transaction_info_missing_token_id() {
        let tx = TransactionInfo {
            tx_hash: "hash123".to_string(),
            event_id: 1,
            chain_id: "chain1".to_string(),
            from: "0xsender".to_string(),
            to: "0xrecipient".to_string(),
            value: None,
            timestamp: 12345,
            token_id: None,
            event_type: EventType::Transfer,
            compliance: crate::interfaces::event::ERCCompliance::OPENZEPPELIN,
            action: crate::interfaces::event::ErcAction::OTHER,
            contract_address: "0xcontract".to_string(),
            contract_type: ContractType::ERC721,
            block_hash: "blockhash".to_string(),
            sub_event_id: "sub123".to_string(),
        };

        let result = TokenEvent::try_from(tx);
        assert!(matches!(result, Err(TokenEventError::MissingTokenId)));
    }
}

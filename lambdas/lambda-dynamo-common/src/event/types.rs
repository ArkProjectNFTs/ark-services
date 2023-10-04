use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{convert, ProviderError};

/// Data of a event.
#[derive(Serialize, Deserialize, Debug)]
pub struct EventData {
    pub block_number: u64,
    pub event_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub from_address: String,
    pub to_address: String,
    pub contract_address: String,
    pub contract_type: String,
    pub token_id: String,
    pub transaction_hash: String,
}

impl TryFrom<HashMap<String, AttributeValue>> for EventData {
    type Error = ProviderError;

    fn try_from(data: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(EventData {
            block_number: convert::attr_to_u64(&data, "BlockNumber")?,
            event_id: convert::attr_to_str(&data, "EventId")?,
            event_type: convert::attr_to_str(&data, "EventType")?,
            timestamp: convert::attr_to_u64(&data, "Timestamp")?,
            from_address: convert::attr_to_str(&data, "FromAddress")?,
            to_address: convert::attr_to_str(&data, "ToAddress")?,
            contract_address: convert::attr_to_str(&data, "ContractAddress")?,
            contract_type: convert::attr_to_str(&data, "ContractType")?,
            token_id: convert::attr_to_str(&data, "TokenId")?,
            transaction_hash: convert::attr_to_str(&data, "TransactionHash")?,
        })
    }
}

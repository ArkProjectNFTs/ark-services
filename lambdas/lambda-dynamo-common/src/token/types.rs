use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{convert, ProviderError};

/// Data of a token.
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    pub block_number: u64,
    pub mint_timestamp: u64,
    pub mint_address: String,
    pub owner: String,
    pub token_id: String,
    pub contract_address: String,
}

impl TryFrom<HashMap<String, AttributeValue>> for TokenData {
    type Error = ProviderError;

    fn try_from(data: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(TokenData {
            block_number: convert::attr_to_u64(&data, "BlockNumber")?,
            mint_timestamp: convert::attr_to_u64(&data, "MintTimestamp")?,
            mint_address: convert::attr_to_str(&data, "MintAddress")?,
            owner: convert::attr_to_str(&data, "Owner")?,
            token_id: convert::attr_to_str(&data, "TokenId")?,
            contract_address: convert::attr_to_str(&data, "ContractAddress")?,
        })
    }
}

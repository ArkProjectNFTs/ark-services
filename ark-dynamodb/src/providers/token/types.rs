use arkproject::metadata::types::TokenMetadata;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{convert, ProviderError};

#[derive(Default, Serialize, Deserialize)]
pub struct TokenData {
    pub contract_address: String,
    pub token_id: String,
    pub token_id_hex: String,
    pub owner: String,
    pub mint_address: Option<String>,
    pub mint_timestamp: Option<u64>,
    pub mint_transaction_hash: Option<String>,
    pub mint_block_number: Option<u64>,
    pub metadata: Option<TokenMetadata>,
}

impl TryFrom<HashMap<String, AttributeValue>> for TokenData {
    type Error = ProviderError;

    fn try_from(data: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let mint_block_number = match convert::attr_to_u64(&data, "MintBlockNumber") {
            Ok(v) => Some(v),
            _ => None,
        };
        let mint_timestamp = match convert::attr_to_u64(&data, "MintTimestamp") {
            Ok(v) => Some(v),
            _ => None,
        };
        let mint_transaction_hash = match convert::attr_to_str(&data, "MintTransactionHash") {
            Ok(v) => Some(v),
            _ => None,
        };
        let mint_address = match convert::attr_to_str(&data, "MintAddress") {
            Ok(v) => Some(v),
            _ => None,
        };

        Ok(TokenData {
            owner: convert::attr_to_str(&data, "Owner")?,
            contract_address: convert::attr_to_str(&data, "ContractAddress")?,
            token_id: convert::attr_to_str(&data, "TokenId")?,
            token_id_hex: convert::attr_to_str(&data, "TokenIdHex")?,
            mint_block_number,
            mint_timestamp,
            mint_transaction_hash,
            mint_address,
            metadata: Some(TokenMetadata {
                image: Some(String::from("")),
                image_data: Some(String::from("")),
                external_url: Some(String::from("")),
                description: Some(String::from("")),
                name: Some(String::from("")),
                attributes: Some(vec![]),
                background_color: Some(String::from("")),
                animation_url: None,
                youtube_url: None,
            }),
        })
    }
}

impl From<&TokenData> for HashMap<String, AttributeValue> {
    fn from(data: &TokenData) -> Self {
        let mut map = HashMap::new();
        map.insert("Owner".to_string(), AttributeValue::S(data.owner.clone()));
        map.insert(
            "ContractAddress".to_string(),
            AttributeValue::S(data.contract_address.clone()),
        );
        map.insert(
            "TokenId".to_string(),
            AttributeValue::S(data.token_id.clone()),
        );
        map.insert(
            "TokenIdHex".to_string(),
            AttributeValue::S(data.token_id_hex.clone()),
        );

        if let Some(v) = data.mint_block_number {
            map.insert(
                "MintBlockNumber".to_string(),
                AttributeValue::N(v.to_string()),
            );
        }
        if let Some(v) = data.mint_timestamp {
            map.insert(
                "MintTimestamp".to_string(),
                AttributeValue::N(v.to_string()),
            );
        }
        if let Some(v) = &data.mint_address {
            map.insert("MintAddress".to_string(), AttributeValue::S(v.clone()));
        }
        if let Some(v) = &data.mint_transaction_hash {
            map.insert(
                "MintTransactionHash".to_string(),
                AttributeValue::S(v.clone()),
            );
        }

        map
    }
}

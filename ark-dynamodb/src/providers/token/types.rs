use crate::{convert, ProviderError};
use arkproject::metadata::types::{MetadataAttributeValue, NormalizedMetadata, TokenMetadata};
use arkproject::pontos::storage::types::TokenMintInfo;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize)]
pub struct TokenData {
    pub contract_address: String,
    pub token_id: String,
    pub token_id_hex: String,
    pub owner: String,
    pub mint_info: Option<TokenMintInfo>,
    pub metadata: Option<TokenMetadata>,
}

impl TokenData {
    pub fn mint_info_to_map(info: &TokenMintInfo) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();

        map.insert(
            "Timestamp".to_string(),
            AttributeValue::N(info.timestamp.to_string()),
        );
        map.insert(
            "Address".to_string(),
            AttributeValue::S(info.address.clone()),
        );
        map.insert(
            "TransactionHash".to_string(),
            AttributeValue::S(info.transaction_hash.clone()),
        );

        map
    }

    pub fn metadata_to_map(meta: &TokenMetadata) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();

        map.insert(
            "Image".to_string(),
            AttributeValue::S(meta.normalized.image.clone().unwrap_or_default()),
        );
        map.insert(
            "ImageData".to_string(),
            AttributeValue::S(meta.normalized.image_data.clone().unwrap_or_default()),
        );
        map.insert(
            "ExternalUrl".to_string(),
            AttributeValue::S(meta.normalized.external_url.clone().unwrap_or_default()),
        );
        map.insert(
            "Description".to_string(),
            AttributeValue::S(meta.normalized.description.clone().unwrap_or_default()),
        );
        map.insert(
            "Name".to_string(),
            AttributeValue::S(meta.normalized.name.clone().unwrap_or_default()),
        );
        map.insert(
            "BackgroundColor".to_string(),
            AttributeValue::S(meta.normalized.background_color.clone().unwrap_or_default()),
        );
        map.insert(
            "AnimationUrl".to_string(),
            AttributeValue::S(meta.normalized.animation_url.clone().unwrap_or_default()),
        );
        map.insert(
            "YoutubeUrl".to_string(),
            AttributeValue::S(meta.normalized.youtube_url.clone().unwrap_or_default()),
        );

        if let Some(attributes) = &meta.normalized.attributes {
            let mut attributes_hashmap: HashMap<String, AttributeValue> = HashMap::new();
            for (_, attribute) in attributes.iter().enumerate() {
                if let Some(display_type) = &attribute.display_type {
                    match display_type {
                        arkproject::metadata::types::DisplayType::Number => {
                            attributes_hashmap.insert(
                                String::from("DisplayType"),
                                AttributeValue::S(String::from("number")),
                            );
                        }
                        arkproject::metadata::types::DisplayType::BoostPercentage => {
                            attributes_hashmap.insert(
                                String::from("DisplayType"),
                                AttributeValue::S(String::from("boost_number")),
                            );
                        }
                        arkproject::metadata::types::DisplayType::BoostNumber => {
                            attributes_hashmap.insert(
                                String::from("DisplayType"),
                                AttributeValue::S(String::from("boost_percentage")),
                            );
                        }
                        arkproject::metadata::types::DisplayType::Date => {
                            attributes_hashmap.insert(
                                String::from("DisplayType"),
                                AttributeValue::S(String::from("date")),
                            );
                        }
                    }
                }

                if let Some(trait_type) = &attribute.trait_type {
                    attributes_hashmap.insert(
                        String::from("TraitType"),
                        AttributeValue::S(trait_type.to_string()),
                    );
                }

                match &attribute.value {
                    MetadataAttributeValue::String(value) => {
                        let attribute_value = AttributeValue::S(value.clone());
                        attributes_hashmap.insert(String::from("Value"), attribute_value);
                    }
                    MetadataAttributeValue::Bool(value) => {
                        let attribute_value = AttributeValue::Bool(value.clone());
                        attributes_hashmap.insert(String::from("Value"), attribute_value);
                    }
                    MetadataAttributeValue::Number(value) => {
                        let attribute_value = AttributeValue::N(value.to_string());
                        attributes_hashmap.insert(String::from("Value"), attribute_value);
                    }
                    _ => {}
                };
            }

            map.insert(
                String::from("Attributes"),
                AttributeValue::M(attributes_hashmap),
            );
        }

        map
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for TokenData {
    type Error = ProviderError;

    fn try_from(data: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let mint_info = match convert::attr_to_map(&data, "MintInfo") {
            Ok(m) => Some(TokenMintInfo {
                address: convert::attr_to_str(&m, "Address")?,
                timestamp: convert::attr_to_u64(&m, "Timestamp")?,
                transaction_hash: convert::attr_to_str(&m, "TransactionHash")?,
            }),
            _ => None,
        };

        let metadata = match convert::attr_to_map(&data, "Metadata") {
            Ok(m) => Some(TokenMetadata {
                normalized: NormalizedMetadata {
                    image: convert::attr_to_opt_str(&m, "Image")?,
                    image_data: convert::attr_to_opt_str(&m, "ImageData")?,
                    external_url: convert::attr_to_opt_str(&m, "ExternalUrl")?,
                    description: convert::attr_to_opt_str(&m, "Description")?,
                    name: convert::attr_to_opt_str(&m, "Name")?,
                    background_color: convert::attr_to_opt_str(&m, "BackgroundColor")?,
                    animation_url: convert::attr_to_opt_str(&m, "AnimationUrl")?,
                    youtube_url: convert::attr_to_opt_str(&m, "YoutubeUrl")?,
                    // TODO: attributes -> Vec of attributes.
                    attributes: None,
                },
                raw: String::from(""),
            }),
            _ => None,
        };

        Ok(TokenData {
            owner: convert::attr_to_str(&data, "Owner")?,
            contract_address: convert::attr_to_str(&data, "ContractAddress")?,
            token_id: convert::attr_to_str(&data, "TokenId")?,
            token_id_hex: convert::attr_to_str(&data, "TokenIdHex")?,
            mint_info,
            metadata,
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

        if let Some(info) = &data.mint_info {
            let mint_map = TokenData::mint_info_to_map(info);
            map.insert("MintInfo".to_string(), AttributeValue::M(mint_map));
        }

        if let Some(meta) = &data.metadata {
            let meta_map = TokenData::metadata_to_map(meta);
            map.insert("Metadata".to_string(), AttributeValue::M(meta_map));
        }

        map
    }
}

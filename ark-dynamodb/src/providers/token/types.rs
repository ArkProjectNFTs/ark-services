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

    pub fn metadata_to_map(metadata: &TokenMetadata) -> HashMap<String, AttributeValue> {
        let mut metadata_map = HashMap::new();

        metadata_map.insert(
            "Image".to_string(),
            AttributeValue::S(metadata.normalized.image.clone().unwrap_or_default()),
        );
        metadata_map.insert(
            "ImageData".to_string(),
            AttributeValue::S(metadata.normalized.image_data.clone().unwrap_or_default()),
        );
        metadata_map.insert(
            "ExternalUrl".to_string(),
            AttributeValue::S(metadata.normalized.external_url.clone().unwrap_or_default()),
        );
        metadata_map.insert(
            "Description".to_string(),
            AttributeValue::S(metadata.normalized.description.clone().unwrap_or_default()),
        );
        metadata_map.insert(
            "Name".to_string(),
            AttributeValue::S(metadata.normalized.name.clone().unwrap_or_default()),
        );
        metadata_map.insert(
            "BackgroundColor".to_string(),
            AttributeValue::S(
                metadata
                    .normalized
                    .background_color
                    .clone()
                    .unwrap_or_default(),
            ),
        );
        metadata_map.insert(
            "AnimationUrl".to_string(),
            AttributeValue::S(
                metadata
                    .normalized
                    .animation_url
                    .clone()
                    .unwrap_or_default(),
            ),
        );
        metadata_map.insert(
            "YoutubeUrl".to_string(),
            AttributeValue::S(metadata.normalized.youtube_url.clone().unwrap_or_default()),
        );

        if let Some(attributes) = &metadata.normalized.attributes {
            let mut attribute_values = vec![];

            for attribute in attributes {
                let mut attribute_map = HashMap::new();

                if let Some(display_type) = &attribute.display_type {
                    attribute_map.insert(
                        String::from("DisplayType"),
                        AttributeValue::S(display_type.to_string()),
                    );
                }

                if let Some(trait_type) = &attribute.trait_type {
                    attribute_map.insert(
                        String::from("TraitType"),
                        AttributeValue::S(trait_type.clone()),
                    );
                }

                match &attribute.value {
                    MetadataAttributeValue::String(value) => {
                        attribute_map.insert("Value".to_string(), AttributeValue::S(value.clone()));
                    }
                    MetadataAttributeValue::Bool(value) => {
                        attribute_map.insert("Value".to_string(), AttributeValue::Bool(*value));
                    }
                    MetadataAttributeValue::BoolVec(values) => {
                        let attribute_values: Vec<AttributeValue> = values
                            .iter()
                            .map(|value| AttributeValue::Bool(*value))
                            .collect();
                        attribute_map
                            .insert("Value".to_string(), AttributeValue::L(attribute_values));
                    }
                    MetadataAttributeValue::Number(value) => {
                        attribute_map
                            .insert("Value".to_string(), AttributeValue::N(value.to_string()));
                    }
                    MetadataAttributeValue::NumberVec(values) => {
                        let attribute_values: Vec<AttributeValue> = values
                            .iter()
                            .map(|value| AttributeValue::N(value.to_string()))
                            .collect();
                        attribute_map
                            .insert("Value".to_string(), AttributeValue::L(attribute_values));
                    }
                    MetadataAttributeValue::StringVec(values) => {
                        let attribute_values: Vec<AttributeValue> = values
                            .iter()
                            .map(|value| AttributeValue::S(value.clone()))
                            .collect();
                        attribute_map
                            .insert("Value".to_string(), AttributeValue::L(attribute_values));
                    }
                };

                attribute_values.push(AttributeValue::M(attribute_map));
            }

            metadata_map.insert(
                "Attributes".to_string(),
                AttributeValue::L(attribute_values),
            );

            metadata_map.insert(
                String::from("RawMetadata"),
                AttributeValue::S(metadata.raw.clone()),
            );
        }

        let mut map: HashMap<String, AttributeValue> = HashMap::new();
        map.insert(
            String::from("RawMetadata"),
            AttributeValue::S(metadata.raw.clone()),
        );
        map.insert(String::from("Metadata"), AttributeValue::M(metadata_map));

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
                raw: convert::attr_to_str(&m, "RawMetadata")?,
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

#[cfg(test)]
#[cfg(test)]
mod tests {
    use arkproject::metadata::types::MetadataAttribute;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_metadata_to_map() {
        let mock_metadata = TokenMetadata {
            raw: "{ \"image\": \"image_url\" }".to_string(),
            normalized: NormalizedMetadata {
                image: Some("image_url".to_string()),
                image_data: Some("image_data".to_string()),
                external_url: Some("external_url".to_string()),
                description: Some("description".to_string()),
                name: Some("name".to_string()),
                background_color: Some("color".to_string()),
                animation_url: Some("animation_url".to_string()),
                youtube_url: Some("youtube_url".to_string()),
                attributes: Some(vec![MetadataAttribute {
                    display_type: None,
                    trait_type: Some("trait".to_string()),
                    value: MetadataAttributeValue::String("value".to_string()),
                }]),
            },
        };

        // Call the function
        let result_map = TokenData::metadata_to_map(&mock_metadata);

        // Build the expected result
        let mut expected_map: HashMap<String, AttributeValue> = HashMap::new();
        expected_map.insert(
            "Image".to_string(),
            AttributeValue::S("image_url".to_string()),
        );
        expected_map.insert(
            "ImageData".to_string(),
            AttributeValue::S("image_data".to_string()),
        );
        expected_map.insert(
            "ExternalUrl".to_string(),
            AttributeValue::S("external_url".to_string()),
        );
        expected_map.insert(
            "Description".to_string(),
            AttributeValue::S("description".to_string()),
        );
        expected_map.insert("Name".to_string(), AttributeValue::S("name".to_string()));
        expected_map.insert(
            "BackgroundColor".to_string(),
            AttributeValue::S("color".to_string()),
        );
        expected_map.insert(
            "AnimationUrl".to_string(),
            AttributeValue::S("animation_url".to_string()),
        );
        expected_map.insert(
            "YoutubeUrl".to_string(),
            AttributeValue::S("youtube_url".to_string()),
        );

        let mut attribute_map = HashMap::new();
        attribute_map.insert(
            "TraitType".to_string(),
            AttributeValue::S("trait".to_string()),
        );
        attribute_map.insert("Value".to_string(), AttributeValue::S("value".to_string()));
        expected_map.insert(
            "Attributes".to_string(),
            AttributeValue::L(vec![AttributeValue::M(attribute_map)]),
        );

        expected_map.insert(
            "RawMetadata".to_string(),
            AttributeValue::S("raw_metadata".to_string()),
        );

        let mut final_expected_map: HashMap<String, AttributeValue> = HashMap::new();
        final_expected_map.insert(
            "RawMetadata".to_string(),
            AttributeValue::S("raw_metadata".to_string()),
        );
        final_expected_map.insert("Metadata".to_string(), AttributeValue::M(expected_map));

        assert_eq!(result_map, final_expected_map);
    }
}

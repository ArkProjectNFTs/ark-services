use crate::{convert, ProviderError};
use anyhow::{anyhow, Result};
use arkproject::metadata::types::{
    DisplayType, MetadataAttribute, MetadataTraitValue, NormalizedMetadata, TokenMetadata,
};
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
        let mut normalized_metadata = HashMap::new();

        normalized_metadata.insert(
            "ImageKey".to_string(),
            AttributeValue::S(metadata.normalized.image_key.clone().unwrap_or_default()),
        );
        normalized_metadata.insert(
            "Image".to_string(),
            AttributeValue::S(metadata.normalized.image.clone().unwrap_or_default()),
        );
        normalized_metadata.insert(
            "ImageData".to_string(),
            AttributeValue::S(metadata.normalized.image_data.clone().unwrap_or_default()),
        );
        normalized_metadata.insert(
            "ImageMimeType".to_string(),
            AttributeValue::S(
                metadata
                    .normalized
                    .image_mime_type
                    .clone()
                    .unwrap_or_default(),
            ),
        );

        normalized_metadata.insert(
            "AnimationKey".to_string(),
            AttributeValue::S(
                metadata
                    .normalized
                    .animation_key
                    .clone()
                    .unwrap_or_default(),
            ),
        );
        normalized_metadata.insert(
            "AnimationMimeType".to_string(),
            AttributeValue::S(
                metadata
                    .normalized
                    .animation_mime_type
                    .clone()
                    .unwrap_or_default(),
            ),
        );
        normalized_metadata.insert(
            "AnimationUrl".to_string(),
            AttributeValue::S(
                metadata
                    .normalized
                    .animation_url
                    .clone()
                    .unwrap_or_default(),
            ),
        );

        normalized_metadata.insert(
            "ExternalUrl".to_string(),
            AttributeValue::S(metadata.normalized.external_url.clone().unwrap_or_default()),
        );
        normalized_metadata.insert(
            "Description".to_string(),
            AttributeValue::S(metadata.normalized.description.clone().unwrap_or_default()),
        );
        normalized_metadata.insert(
            "Name".to_string(),
            AttributeValue::S(metadata.normalized.name.clone().unwrap_or_default()),
        );
        normalized_metadata.insert(
            "BackgroundColor".to_string(),
            AttributeValue::S(
                metadata
                    .normalized
                    .background_color
                    .clone()
                    .unwrap_or_default(),
            ),
        );

        normalized_metadata.insert(
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
                    MetadataTraitValue::String(value) => {
                        attribute_map.insert("Value".to_string(), AttributeValue::S(value.clone()));
                    }
                    MetadataTraitValue::Number(value) => {
                        attribute_map
                            .insert("Value".to_string(), AttributeValue::N(value.to_string()));
                    }
                    MetadataTraitValue::Array(value) => {
                        attribute_map.insert(
                            "Value".to_string(),
                            AttributeValue::L(
                                value.iter().map(|s| AttributeValue::S(s.clone())).collect(),
                            ),
                        );
                    }
                    MetadataTraitValue::Boolean(value) => {
                        attribute_map.insert("Value".to_string(), AttributeValue::Bool(*value));
                    }
                };
                attribute_values.push(AttributeValue::M(attribute_map));
            }

            normalized_metadata.insert(
                "Attributes".to_string(),
                AttributeValue::L(attribute_values),
            );
        }

        let mut map: HashMap<String, AttributeValue> = HashMap::new();
        map.insert(
            String::from("RawMetadata"),
            AttributeValue::S(metadata.raw.clone()),
        );
        map.insert(
            String::from("NormalizedMetadata"),
            AttributeValue::M(normalized_metadata),
        );

        map
    }
}

fn extract_attributes_from_hashmap(
    map: HashMap<String, AttributeValue>,
) -> Result<Vec<MetadataAttribute>> {
    if let Some(AttributeValue::L(attributes_list_av)) = map.get("Attributes") {
        let mut attributes: Vec<MetadataAttribute> = vec![];
        for attr_value_map in attributes_list_av {
            if attr_value_map.is_m() {
                let attr_value = attr_value_map.as_m().unwrap();
                let display_type_str = convert::attr_to_opt_str(attr_value, "DisplayType")?;
                let display_type = display_type_str.map(|s| match s.as_str() {
                    "number" => DisplayType::Number,
                    "boost_number" => DisplayType::BoostNumber,
                    "boost_percentage" => DisplayType::BoostPercentage,
                    "date" => DisplayType::Date,
                    _ => DisplayType::Number,
                });
                let trait_type = convert::attr_to_opt_str(attr_value, "TraitType")?;

                let value = if let Some(attribute_value) = attr_value.get("Value") {
                    if attribute_value.is_s() {
                        MetadataTraitValue::String(attribute_value.as_s().unwrap().to_string())
                    } else {
                        MetadataTraitValue::String(String::from(""))
                    }
                } else {
                    MetadataTraitValue::String(String::from(""))
                };

                let metadata_attribute = MetadataAttribute {
                    display_type,
                    trait_type,
                    value,
                };

                attributes.push(metadata_attribute);
            }
        }
        return Ok(attributes);
    }
    Err(anyhow!("Attributes not found"))
}

fn extract_normalized_metadata_from_hashmap(
    map: HashMap<String, AttributeValue>,
) -> Result<NormalizedMetadata> {
    match map.get("NormalizedMetadata") {
        Some(AttributeValue::M(normalized_metadata_hashmap)) => {
            let attributes =
                match extract_attributes_from_hashmap(normalized_metadata_hashmap.clone()) {
                    Ok(attributes) => Some(attributes),
                    _ => None,
                };

            let normalized_metadata = NormalizedMetadata {
                image_key: convert::attr_to_opt_str(normalized_metadata_hashmap, "ImageKey")?,
                image_mime_type: convert::attr_to_opt_str(
                    normalized_metadata_hashmap,
                    "ImageMimeType",
                )?,
                image: convert::attr_to_opt_str(normalized_metadata_hashmap, "Image")?,
                image_data: convert::attr_to_opt_str(normalized_metadata_hashmap, "ImageData")?,
                animation_key: convert::attr_to_opt_str(
                    normalized_metadata_hashmap,
                    "AnimationKey",
                )?,
                animation_mime_type: convert::attr_to_opt_str(
                    normalized_metadata_hashmap,
                    "AnimationMimeType",
                )?,
                animation_url: convert::attr_to_opt_str(
                    normalized_metadata_hashmap,
                    "AnimationUrl",
                )?,
                external_url: convert::attr_to_opt_str(normalized_metadata_hashmap, "ExternalUrl")?,
                description: convert::attr_to_opt_str(normalized_metadata_hashmap, "Description")?,
                name: convert::attr_to_opt_str(normalized_metadata_hashmap, "Name")?,
                background_color: convert::attr_to_opt_str(
                    normalized_metadata_hashmap,
                    "BackgroundColor",
                )?,
                youtube_url: convert::attr_to_opt_str(normalized_metadata_hashmap, "YoutubeUrl")?,
                attributes,
                properties: None, // TODO
            };

            Ok(normalized_metadata)
        }
        _ => Err(anyhow!("NormalizedMetadata not found")),
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

        let metadata = if data.get("Metadata").is_some() {
            let m = convert::attr_to_map(&data, "Metadata")?;

            let normalized_metadata =
                extract_normalized_metadata_from_hashmap(m.clone()).map_err(|_| {
                    ProviderError::DataValueError(String::from(
                        "Extracting normalized metadata from hashmap failed",
                    ))
                })?;

            let raw_metadata = match m.get("RawMetadata") {
                Some(AttributeValue::S(s)) => s.clone(),
                _ => String::from(""),
            };

            let metadata_updated_at = match m.get("MetadataUpdatedAt") {
                Some(AttributeValue::N(n)) => match n.parse::<i64>() {
                    Ok(n) => Some(n),
                    _ => None,
                },
                _ => None,
            };

            Some(TokenMetadata {
                raw: raw_metadata.clone(),
                normalized: normalized_metadata,
                metadata_updated_at,
            })
        } else {
            None
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
mod tests {
    use super::*;
    use arkproject::metadata::types::MetadataAttribute;
    use std::collections::HashMap;

    #[test]
    fn test_mint_info_to_map() {
        let mock_mint_info = TokenMintInfo {
            address: "0x1234".to_string(),
            timestamp: 12345678,
            transaction_hash: "0x5678".to_string(),
        };

        let result_map = TokenData::mint_info_to_map(&mock_mint_info);

        let mut expected_map = HashMap::new();
        expected_map.insert(
            "Timestamp".to_string(),
            AttributeValue::N("12345678".to_string()),
        );
        expected_map.insert(
            "Address".to_string(),
            AttributeValue::S("0x1234".to_string()),
        );
        expected_map.insert(
            "TransactionHash".to_string(),
            AttributeValue::S("0x5678".to_string()),
        );

        assert_eq!(result_map, expected_map);
    }

    #[test]
    fn test_try_from() {
        let mock_data = {
            let mut map = HashMap::new();
            map.insert(
                "Owner".to_string(),
                AttributeValue::S("owner_address".to_string()),
            );
            map.insert(
                "ContractAddress".to_string(),
                AttributeValue::S("contract_address".to_string()),
            );
            map.insert(
                "TokenId".to_string(),
                AttributeValue::S("token_id".to_string()),
            );
            map.insert(
                "TokenIdHex".to_string(),
                AttributeValue::S("token_id_hex".to_string()),
            );

            let mut metadata = HashMap::new();
            metadata.insert(
                "RawMetadata".to_string(),
                AttributeValue::S("{ \"image\": \"image_url\" }".to_string()),
            );

            let mut normalized_metadata = HashMap::new();
            normalized_metadata.insert(
                "Image".to_string(),
                AttributeValue::S("image_url".to_string()),
            );

            metadata.insert(
                "NormalizedMetadata".to_string(),
                AttributeValue::M(normalized_metadata),
            );

            map.insert("Metadata".to_string(), AttributeValue::M(metadata));

            map
        };

        let token_data = TokenData::try_from(mock_data).unwrap();

        assert_eq!(token_data.owner, "owner_address");
        let metadata_result = token_data.metadata.unwrap();
        assert_eq!(metadata_result.raw, "{ \"image\": \"image_url\" }");

        println!(
            "metadata_result.normalized: {:?}",
            metadata_result.normalized
        );
        assert_eq!(metadata_result.normalized.image.unwrap(), "image_url");
    }

    #[test]
    fn test_from() {
        let mock_token_data = TokenData {
            owner: "0x0131E3134b75c5fB3B7c0BDdDF4289625d0030F796c4F9D30dB45da472574199".to_string(),
            contract_address: "contract_address".to_string(),
            token_id: "token_id".to_string(),
            token_id_hex: "token_id_hex".to_string(),
            mint_info: Some(TokenMintInfo {
                address: String::from(
                    "0x0131E3134b75c5fB3B7c0BDdDF4289625d0030F796c4F9D30dB45da472574199",
                ),
                timestamp: 1698237736,
                transaction_hash: String::from("0x01"),
            }),
            metadata: Some(TokenMetadata {
                normalized: NormalizedMetadata {
                    animation_key: Some(String::from("animation_key")),
                    animation_mime_type: Some(String::from("video/mp4")),
                    image_key: Some(String::from("image_key")),
                    image_mime_type: Some(String::from("image/png")),
                    image: Some(String::from("image_url")),
                    image_data: Some(String::from("")),
                    external_url: Some(String::from("")),
                    description: Some(String::from("")),
                    name: Some(String::from("")),
                    attributes: Some(vec![]),
                    properties: None,
                    background_color: Some(String::from("")),
                    animation_url: Some(String::from("")),
                    youtube_url: Some(String::from("")),
                },
                raw: String::from("{ \"image\": \"image_url\" }"),
                metadata_updated_at: None,
            }),
        };

        let result_map: HashMap<String, AttributeValue> = (&mock_token_data).into();
        let owner = result_map.get("Owner").unwrap().as_s().unwrap();

        assert_eq!(
            owner,
            "0x0131E3134b75c5fB3B7c0BDdDF4289625d0030F796c4F9D30dB45da472574199"
        );

        let metadata = result_map.get("Metadata").unwrap().as_m().unwrap();
        let raw_metadata = metadata.get("RawMetadata").unwrap().as_s().unwrap();
        let normalized_metadata = metadata.get("NormalizedMetadata").unwrap().as_m().unwrap();

        assert_eq!(raw_metadata, "{ \"image\": \"image_url\" }");
        assert_eq!(
            normalized_metadata.get("Image").unwrap().as_s().unwrap(),
            &"image_url"
        );
    }

    #[test]
    fn test_metadata_to_map() {
        let mock_metadata = TokenMetadata {
            normalized: NormalizedMetadata {
                animation_key: Some(String::from("animation_key")),
                animation_mime_type: Some(String::from("video/mp4")),
                image_key: Some(String::from("image_key")),
                image_mime_type: Some(String::from("image/png")),
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
                    value: MetadataTraitValue::String("value".to_string()),
                }]),

                properties: None,
            },
            raw: "{ \"image\": \"image_url\" }".to_string(),
            metadata_updated_at: None,
        };

        // Call the function
        let result_map = TokenData::metadata_to_map(&mock_metadata);

        // Build the expected result
        let mut attribute_map = HashMap::new();
        attribute_map.insert(
            "TraitType".to_string(),
            AttributeValue::S("trait".to_string()),
        );
        attribute_map.insert("Value".to_string(), AttributeValue::S("value".to_string()));

        let mut expected_map = HashMap::new();

        expected_map.insert(
            "Description".to_string(),
            AttributeValue::S("description".to_string()),
        );

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
        expected_map.insert(
            "Attributes".to_string(),
            AttributeValue::L(vec![AttributeValue::M(attribute_map)]),
        );

        let mut final_expected_map = HashMap::new();

        final_expected_map.insert("Metadata".to_string(), AttributeValue::M(expected_map));

        final_expected_map.insert(
            "RawMetadata".to_string(),
            AttributeValue::S("{ \"image\": \"image_url\" }".to_string()),
        );

        final_expected_map.insert(
            "MetadataUpdatedAt".to_string(),
            AttributeValue::N("1698346591".to_string()),
        );

        assert_eq!(result_map.get("Image"), final_expected_map.get("Image"));

        assert_eq!(
            result_map.get("Description"),
            final_expected_map.get("Description")
        );
    }
}

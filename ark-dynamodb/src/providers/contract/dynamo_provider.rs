use super::{ArkContractProvider, ContractInfo};
use crate::{convert, DynamoDbCtx, DynamoDbOutput, EntityType, ProviderError};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, ReturnConsumedCapacity, Select};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use tracing::{debug, error, info, trace, warn};

/// DynamoDB provider for contracts.
pub struct DynamoDbContractProvider {
    table_name: String,
    key_prefix: String,
    limit: Option<i32>,
}

impl DynamoDbContractProvider {
    pub fn new(table_name: &str, limit: Option<i32>) -> Self {
        DynamoDbContractProvider {
            table_name: table_name.to_string(),
            key_prefix: "CONTRACT".to_string(),
            limit,
        }
    }

    fn get_pk(&self, contract_address: &str) -> String {
        format!("{}#{}", self.key_prefix, contract_address)
    }

    fn get_sk(&self) -> String {
        self.key_prefix.clone()
    }

    pub fn data_to_info(
        data: &HashMap<String, AttributeValue>,
    ) -> Result<ContractInfo, ProviderError> {
        let get_attr_or_default =
            |key: &str| -> String { convert::attr_to_str(data, key).unwrap_or_default() };

        Ok(ContractInfo {
            contract_type: convert::attr_to_str(data, "ContractType")?,
            contract_address: convert::attr_to_str(data, "ContractAddress")?,
            name: Some(get_attr_or_default("Name")),
            symbol: Some(get_attr_or_default("Symbol")),
            image: Some(get_attr_or_default("Image")),
        })
    }

    pub fn info_to_data(info: &ContractInfo) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            "ContractType".to_string(),
            AttributeValue::S(info.contract_type.to_string()),
        );
        map.insert(
            "ContractAddress".to_string(),
            AttributeValue::S(info.contract_address.clone()),
        );

        if let Some(name) = info.name.clone() {
            map.insert("Name".to_string(), AttributeValue::S(name));
        }

        if let Some(symbol) = info.symbol.clone() {
            map.insert("Symbol".to_string(), AttributeValue::S(symbol));
        }

        if let Some(image) = info.image.clone() {
            map.insert("Image".to_string(), AttributeValue::S(image));
        }

        map
    }
}

#[async_trait]
impl ArkContractProvider for DynamoDbContractProvider {
    type Client = DynamoClient;

    async fn register_contract(
        &self,
        ctx: &DynamoDbCtx,
        info: &ContractInfo,
        block_timestamp: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        trace!("register_contract called with info: {:?}", info);

        let pk = self.get_pk(&info.contract_address);
        let sk = self.get_sk();

        let data = Self::info_to_data(info);

        debug!("Registering contract with PK: {} and SK: {}", pk, sk);

        let gsi2pk = match info.contract_type.as_str() {
            "ERC721" => String::from("NFT"),
            "ERC1155" => String::from("NFT"),
            _ => String::from("OTHER"),
        };

        let r = ctx
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(pk.clone()))
            .item("SK", AttributeValue::S(sk.clone()))
            // We can't filter on PK/SK to only get all contract, as the PK
            // is required. So we duplicate info in the GSI1. TODO: investiagte more.
            .item("GSI1PK".to_string(), AttributeValue::S(sk.clone()))
            .item("GSI1SK".to_string(), AttributeValue::S(pk.clone()))
            .item("GSI2PK".to_string(), AttributeValue::S(gsi2pk))
            .item(
                "GSI2SK".to_string(),
                AttributeValue::S(block_timestamp.to_string()),
            )
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_timestamp)),
            )
            .item("GSI4SK".to_string(), AttributeValue::S(pk.clone()))
            .item(
                "GSI5PK".to_string(),
                AttributeValue::S("COMPLETE_CONTRACT_DATA#false".to_string()),
            )
            .item(
                "GSI5SK".to_string(),
                AttributeValue::S(format!("CONTRACT#{}", info.contract_address)),
            )
            .item("Data", AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Contract.to_string()))
            .condition_expression("attribute_not_exists(PK)")
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| {
                debug!("Database error while registering contract: {:?}", e);
                ProviderError::DatabaseError(format!("{:?}", e))
            })?;

        debug!("Database operation successful with result: {:?}", r);

        Ok(().into())
    }

    async fn get_batch_contracts(
        &self,
        ctx: &DynamoDbCtx,
        contract_addresses: Vec<String>,
    ) -> Result<DynamoDbOutput<Vec<ContractInfo>>, ProviderError> {
        let mut keys = Vec::new();
        for address in contract_addresses {
            let mut key = HashMap::new();
            key.insert("PK".to_string(), AttributeValue::S(self.get_pk(&address)));
            key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));
            keys.push(key);
        }

        let keys_and_attributes_result = KeysAndAttributes::builder()
            .set_keys(Some(keys.clone()))
            .build();

        match keys_and_attributes_result {
            Ok(keys_and_attributes) => {
                let batch_request_output = ctx
                    .client
                    .batch_get_item()
                    .request_items(self.table_name.clone(), keys_and_attributes)
                    .return_consumed_capacity(ReturnConsumedCapacity::Total)
                    .send()
                    .await
                    .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

                let mut contract_infos = Vec::new();
                if let Some(responses) = batch_request_output.responses {
                    for item in responses.get(&self.table_name).unwrap_or(&Vec::new()) {
                        let data = convert::attr_to_map(item, "Data")?;
                        contract_infos.push(Self::data_to_info(&data)?);
                    }
                }

                // let consumed_capacity_units = batch_request_output.consumed_capacity.map(|c| c.capacity());
                Ok(DynamoDbOutput::new(contract_infos, None, None))
            }
            Err(e) => {
                error!("Error building query. Error: {:?}. Keys: {:?}", e, keys);
                Err(ProviderError::QueryError(
                    "Error building query with keys".to_string(),
                ))
            }
        }
    }

    async fn get_contract(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
    ) -> Result<DynamoDbOutput<Option<ContractInfo>>, ProviderError> {
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(contract_address)),
        );
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let get_item_output = ctx
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let consumed_capacity_units = match get_item_output.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        if let Some(item) = &get_item_output.item {
            let data = convert::attr_to_map(item, "Data")?;
            Ok(DynamoDbOutput::new(
                Some(Self::data_to_info(&data)?),
                consumed_capacity_units,
                None,
            ))
        } else {
            Ok(DynamoDbOutput::new(None, consumed_capacity_units, None))
        }
    }

    async fn update_nft_contract_image(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
    ) -> Result<Option<String>, ProviderError> {
        debug!(
            "Starting update_nft_contract_image for contract address: {}",
            contract_address
        );

        let initial_token_query_response = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI1PK-GSI1SK-index")
            .key_condition_expression("#GSI1PK = :gsi1pk AND begins_with(#GSI1SK, :gsi1sk)")
            .expression_attribute_names("#GSI1PK", "GSI1PK")
            .expression_attribute_names("#GSI1SK", "GSI1SK")
            .expression_attribute_names("#Data", "Data")
            .expression_attribute_names("#Metadata", "Metadata")
            .expression_attribute_names("#NormalizedMetadata", "NormalizedMetadata")
            .expression_attribute_names("#Image", "Image")
            .expression_attribute_names("#GSI5PK", "GSI5PK")
            .expression_attribute_values(
                ":gsi1pk",
                AttributeValue::S(format!("CONTRACT#{}", contract_address)),
            )
            .expression_attribute_values(":gsi1sk", AttributeValue::S(String::from("TOKEN")))
            .expression_attribute_values(
                ":gsi5pk",
                AttributeValue::S(String::from("METADATA#true")),
            )
            .select(Select::SpecificAttributes)
            .projection_expression("#Data.#Metadata.#NormalizedMetadata.#Image")
            .filter_expression("#GSI5PK = :gsi5pk")
            .limit(1)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        if let Some(items) = initial_token_query_response.items {
            if let Some(item) = items.first() {
                info!("Item: {:?}", item);

                let data = convert::attr_to_map(item, "Data").unwrap_or_default();
                let metadata = convert::attr_to_map(&data, "Metadata").unwrap_or_default();
                let normalized_metadata =
                    convert::attr_to_map(&metadata, "NormalizedMetadata").unwrap_or_default();

                if let Ok(image) = convert::attr_to_str(&normalized_metadata, "Image") {
                    if !image.is_empty() && (image.contains("http") || image.contains("ipfs")) {
                        info!("Updating contract image: {:?}", image);

                        let update_result = ctx
                            .client
                            .update_item()
                            .table_name(self.table_name.clone())
                            .key(
                                "PK",
                                AttributeValue::S(format!("CONTRACT#{}", contract_address)),
                            )
                            .key("SK", AttributeValue::S(String::from("CONTRACT")))
                            .update_expression("SET #Data.#Image = :image")
                            .expression_attribute_names("#Data", "Data")
                            .expression_attribute_names("#Image", "Image")
                            .expression_attribute_values(":image", AttributeValue::S(image.clone()))
                            .send()
                            .await;

                        match update_result {
                            Ok(_) => info!(
                                "Image URL updated successfully for contract: {}",
                                contract_address
                            ),
                            Err(e) => {
                                error!(
                                    "Error updating image URL for contract: {}. Error: {:?}",
                                    contract_address, e
                                );
                                return Err(ProviderError::DatabaseError(format!("{:?}", e)));
                            }
                        }

                        return Ok(Some(image));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_nft_contracts(
        &self,
        ctx: &DynamoDbCtx,
    ) -> Result<DynamoDbOutput<Vec<ContractInfo>>, ProviderError> {
        trace!("get_nft_contracts");

        let mut values = HashMap::new();
        values.insert(":pk".to_string(), AttributeValue::S("NFT".to_string()));
        values.insert(":name".to_string(), AttributeValue::S("Sheet".to_string()));

        let collections_query_output = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some("GSI2PK = :pk".to_string()))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .expression_attribute_names("#Data", "Data")
            .expression_attribute_names("#Name", "Name")
            .filter_expression("NOT contains(#Data.#Name, :name)".to_string())
            .set_limit(self.limit)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let consumed_capacity_units: f64 = collections_query_output
            .consumed_capacity()
            .and_then(|c| c.capacity_units())
            .unwrap_or(0.0);

        let mut res = vec![];
        if let Some(items) = collections_query_output.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                let mut contract_info = Self::data_to_info(&data)?;

                if contract_info.image.is_none() {
                    match self
                        .update_nft_contract_image(ctx, &contract_info.contract_address)
                        .await
                    {
                        Ok(image) => {
                            contract_info.image = image;
                        }
                        Err(e) => {
                            warn!(
                                "Error while fetching and updating collection image: {:?}",
                                e
                            );
                        }
                    }
                }

                res.push(contract_info);
            }
        }

        Ok(DynamoDbOutput::new_lek(
            res,
            Some(consumed_capacity_units),
            collections_query_output.last_evaluated_key,
            None,
        ))
    }
}

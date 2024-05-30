use super::types::{
    BasicTokenData, BatchTokenData, ContractData, ContractWithTokens, TokensParams,
};
use crate::providers::token::types::TokenData;
use crate::providers::ArkTokenProvider;
use crate::{DynamoDbCtx, DynamoDbOutput, EntityType, ProviderError};
use arkproject::metadata::types::TokenMetadata;
use arkproject::pontos::storage::types::TokenMintInfo;
use arkproject::starknet::CairoU256;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, ReturnConsumedCapacity, Select};
use aws_sdk_dynamodb::Client as DynamoClient;
use chrono::Utc;
use std::collections::HashMap;
use std::convert::TryInto;
use tracing::{debug, error, trace};

/// DynamoDB provider for tokens.
pub struct DynamoDbTokenProvider {
    table_name: String,
    key_prefix: String,
    limit: Option<i32>,
}

impl DynamoDbTokenProvider {
    pub fn new(table_name: &str, limit: Option<i32>) -> Self {
        DynamoDbTokenProvider {
            table_name: table_name.to_string(),
            key_prefix: "TOKEN".to_string(),
            limit,
        }
    }

    fn get_pk(&self, contract_address: &str, token_id: &str) -> String {
        format!("{}#{}#{}", self.key_prefix, contract_address, token_id)
    }

    fn get_sk(&self) -> String {
        self.key_prefix.clone()
    }
}

#[async_trait]
impl ArkTokenProvider for DynamoDbTokenProvider {
    type Client = DynamoClient;

    async fn update_owner(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
        owner: &str,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let pk = self.get_pk(contract_address, token_id_hex);
        trace!("Updating owner for token: {}", pk);

        let sk = self.get_sk();

        let _r = ctx
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            .update_expression("SET #data.#owner = :owner, #GSI2PK = :GSI2PK")
            .expression_attribute_names("#data", "Data")
            .expression_attribute_names("#GSI2PK", "GSI2PK")
            .expression_attribute_names("#owner", "Owner")
            .expression_attribute_values(":owner".to_string(), AttributeValue::S(owner.to_string()))
            .expression_attribute_values(
                ":GSI2PK".to_string(),
                AttributeValue::S(format!("OWNER#{}", owner)),
            )
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        Ok(().into())
    }

    async fn update_mint_info(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
        info: &TokenMintInfo,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let pk = self.get_pk(contract_address, token_id_hex);
        let sk = self.get_sk();
        let data = TokenData::mint_info_to_map(info);

        let mut names = HashMap::new();
        names.insert("#data".to_string(), "Data".to_string());
        names.insert("#mint_info".to_string(), "MintInfo".to_string());

        let _r = ctx
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            .update_expression("SET #data.#mint_info = :info")
            .set_expression_attribute_names(Some(names))
            .expression_attribute_values(":info", AttributeValue::M(data))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        Ok(().into())
    }

    async fn update_token_metadata_status(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
        metadata_status: &str,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        trace!(
            "Updating token metadata status for contract address: {}, token_id: {}, metadata_status={}",
            contract_address,
            token_id_hex,
            metadata_status
        );

        let pk = self.get_pk(contract_address, token_id_hex);
        let sk = self.get_sk();

        debug!(
            "Generated primary key (PK): {} and sort key (SK): {} for updating metadata status.",
            pk, sk
        );

        let r = ctx
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk.clone()))
            .key("SK".to_string(), AttributeValue::S(sk.clone()))
            .update_expression("SET GSI5PK = :meta_state")
            .expression_attribute_values(
                ":meta_state",
                AttributeValue::S(format!("METADATA#{}", metadata_status)),
            )
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| {
                debug!(
                    "Database error while updating token metadata status: {:?}",
                    e
                );
                ProviderError::DatabaseError(format!("{:?}", e))
            })?;

        debug!(
            "Database update operation for token metadata status was successful with result: {:?}",
            r
        );

        Ok(().into())
    }

    async fn update_metadata(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
        metadata: &TokenMetadata,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let pk = self.get_pk(contract_address, token_id_hex);
        let sk = self.get_sk();

        trace!("Updating metadata for token: PK={}, SK={}", pk, sk);

        let mut data = TokenData::metadata_to_map(metadata);
        let now = Utc::now();
        let timestamp = now.timestamp();
        data.insert(
            "MetadataUpdatedAt".to_string(),
            AttributeValue::N(timestamp.to_string()),
        );

        let mut names = HashMap::new();
        names.insert("#data".to_string(), "Data".to_string());
        names.insert("#metadata".to_string(), "Metadata".to_string());

        let _r = ctx
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            .update_expression("SET #data.#metadata = :meta, GSI5PK = :meta_state")
            .set_expression_attribute_names(Some(names))
            .expression_attribute_values(":meta", AttributeValue::M(data))
            .expression_attribute_values(
                ":meta_state",
                AttributeValue::S("METADATA#true".to_string()),
            )
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        Ok(().into())
    }

    async fn get_last_refresh_token_metadata(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
    ) -> Result<Option<i64>, ProviderError> {
        let pk = self.get_pk(contract_address, token_id_hex);

        debug!("get_token: pk={}", pk);

        let mut key = HashMap::new();
        key.insert("PK".to_string(), AttributeValue::S(pk));
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let r = ctx
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        if let Some(item) = &r.item {
            if let Some(data) = item.get("Data") {
                if data.is_m() {
                    let data_m = data.as_m().unwrap();

                    if let Some(metadata_updated_at_av) = data_m.get("MetadataUpdatedAt") {
                        if metadata_updated_at_av.is_n() {
                            let metadata_updated_at = metadata_updated_at_av
                                .as_n()
                                .unwrap()
                                .parse::<i64>()
                                .unwrap();

                            return Ok(Some(metadata_updated_at));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_token(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
    ) -> Result<DynamoDbOutput<Option<TokenData>>, ProviderError> {
        let pk = self.get_pk(contract_address, token_id_hex);

        debug!("get_token: pk={}", pk);

        let mut key = HashMap::new();
        key.insert("PK".to_string(), AttributeValue::S(pk));
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let r = ctx
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let consumed_capacity_units = r.consumed_capacity().and_then(|c| c.capacity_units);
        if let Some(item) = &r.item {
            let token_data: TokenData = item.clone().try_into()?;
            Ok(DynamoDbOutput::new(
                Some(token_data),
                consumed_capacity_units,
                None,
            ))
        } else {
            Ok(DynamoDbOutput::new(None, consumed_capacity_units, None))
        }
    }

    async fn get_token_without_metadata(
        &self,
        client: &Self::Client,
        filter: Option<(String, String)>,
    ) -> Result<Vec<(String, String, String)>, ProviderError> {
        let sort_key = match filter {
            Some((contract_address, _chain_id)) => {
                format!("CONTRACT#{}", contract_address.clone())
            }
            None => "CONTRACT".to_string(),
        };
        let query_result = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI5PK-GSI5SK-index")
            .key_condition_expression("GSI5PK = :gsi_pk AND begins_with(GSI5SK, :gsi_sk)")
            .expression_attribute_values(
                ":gsi_pk",
                AttributeValue::S(String::from("METADATA#TO_REFRESH")),
            )
            .expression_attribute_values(":gsi_sk", AttributeValue::S(sort_key))
            .scan_index_forward(false)
            .send()
            .await;

        match query_result {
            Ok(query_output) => {
                if query_output.items.is_none() {
                    return Ok(vec![]);
                }

                let mut results: Vec<(String, String, String)> = Vec::new();
                let items = query_output.items.unwrap();

                for item in items.iter() {
                    if let Some(data) = item.get("Data") {
                        if data.is_m() {
                            let data_m = data.as_m().unwrap();

                            // Extracting contract address
                            match data_m.get("ContractAddress") {
                                Some(AttributeValue::S(contract_address)) => {
                                    // Extracting token ID
                                    if let Some(AttributeValue::S(token_id_attribute_value)) =
                                        data_m.get("TokenIdHex")
                                    {
                                        let cairo_u256_result =
                                            CairoU256::from_hex_be(token_id_attribute_value);

                                        match cairo_u256_result {
                                            Ok(token_id) => {
                                                let bn = token_id.to_biguint();
                                                let token_id_str = bn.to_str_radix(10);

                                                results.push((
                                                    contract_address.to_string(),
                                                    token_id_str,
                                                    "SN_MAIN".to_string(),
                                                ));
                                            }
                                            Err(_) => {
                                                return Err(ProviderError::DataValueError(
                                                    format!(
                                                        "Failed to parse token ID from: {}",
                                                        token_id_attribute_value
                                                    ),
                                                ));
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    return Err(ProviderError::DataValueError(
                                        "ContractAddress attribute not found or not a string."
                                            .to_string(),
                                    ));
                                }
                            }
                        } else {
                            return Err(ProviderError::DataValueError(
                                "Data attribute is not a map.".to_string(),
                            ));
                        }
                    } else {
                        return Err(ProviderError::DataValueError(
                            "Data attribute missing.".to_string(),
                        ));
                    }
                }
                return Ok(results);
            }
            Err(_) => {
                return Err(ProviderError::DatabaseError(String::from(
                    "Failed to query DynamoDB",
                )));
            }
        }
    }

    async fn register_token(
        &self,
        ctx: &DynamoDbCtx,
        info: &TokenData,
        block_timestamp: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let pk = self.get_pk(&info.contract_address, &info.token_id_hex);

        trace!("Puting item in dynamo db: {}", pk);

        let _r = ctx
            .client
            .put_item()
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .table_name(self.table_name.clone())
            .item("PK".to_string(), AttributeValue::S(pk.clone()))
            .item("SK".to_string(), AttributeValue::S(self.get_sk()))
            .item("Type".to_string(), AttributeValue::S("Token".to_string()))
            .item(
                "GSI1PK".to_string(),
                AttributeValue::S(format!("CONTRACT#{}", info.contract_address)),
            )
            .item(
                "GSI1SK".to_string(),
                AttributeValue::S(format!("TOKEN#{}", info.token_id_hex)),
            )
            .item(
                "GSI2PK".to_string(),
                AttributeValue::S(format!("OWNER#{}", info.owner)),
            )
            .item("GSI2SK".to_string(), AttributeValue::S(pk.clone()))
            .item(
                "GSI3PK".to_string(),
                AttributeValue::S("LISTED#false".to_string()),
            )
            .item("GSI3SK".to_string(), AttributeValue::S(pk.clone()))
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_timestamp)),
            )
            .item("GSI4SK".to_string(), AttributeValue::S(pk.clone()))
            .item(
                "GSI5PK".to_string(),
                AttributeValue::S("METADATA#TO_REFRESH".to_string()),
            )
            .item(
                "GSI5SK".to_string(),
                AttributeValue::S(format!(
                    "CONTRACT#{}#{}",
                    info.contract_address, block_timestamp
                )),
            )
            .item("Data".to_string(), AttributeValue::M(info.into()))
            .item("Type", AttributeValue::S(EntityType::Token.to_string()))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        Ok(().into())
    }

    #[allow(clippy::assigning_clones)]
    async fn get_batch_tokens(
        &self,
        ctx: &DynamoDbCtx,
        token_params: Vec<TokensParams>,
    ) -> Result<DynamoDbOutput<Vec<BatchTokenData>>, ProviderError> {
        let mut keys = Vec::new();
        let mut collection_keys = Vec::new();

        for token_param in token_params {
            let mut collection_key = HashMap::new();
            let mut key = HashMap::new();

            key.insert(
                "PK".to_string(),
                AttributeValue::S(format!(
                    "TOKEN#{}#{}",
                    token_param.contract_address, token_param.token_id
                )),
            );
            key.insert("SK".to_string(), AttributeValue::S(String::from("TOKEN")));
            keys.push(key);

            collection_key.insert(
                "PK".to_string(),
                AttributeValue::S(format!("CONTRACT#{}", token_param.contract_address)),
            );
            collection_key.insert(
                "SK".to_string(),
                AttributeValue::S(String::from("CONTRACT")),
            );

            if !collection_keys.contains(&collection_key) {
                collection_keys.push(collection_key);
            }
        }

        let collection_keys_and_attributes_result = KeysAndAttributes::builder()
            .set_keys(Some(collection_keys))
            .build();

        if collection_keys_and_attributes_result.is_err() {
            let e = collection_keys_and_attributes_result.err().unwrap();
            error!("Error building query. Error: {:?}. Keys: {:?}", e, keys);

            return Err(ProviderError::QueryError(
                "Failed to build keys and attributes".to_string(),
            ));
        }

        let collection_keys_and_attributes = collection_keys_and_attributes_result.unwrap();
        let collections_request_output = ctx
            .client
            .batch_get_item()
            .request_items(self.table_name.clone(), collection_keys_and_attributes)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let keys_and_attributes_result = KeysAndAttributes::builder()
            .set_keys(Some(keys.clone()))
            .build();

        if keys_and_attributes_result.is_err() {
            let e = keys_and_attributes_result.err().unwrap();
            error!("Error building query. Error: {:?}. Keys: {:?}", e, keys);

            return Err(ProviderError::QueryError(
                "Failed to build keys and attributes".to_string(),
            ));
        }
        let keys_and_attributes = keys_and_attributes_result.unwrap();

        let batch_request_output = ctx
            .client
            .batch_get_item()
            .request_items(self.table_name.clone(), keys_and_attributes)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut token_results: Vec<BatchTokenData> = Vec::new();
        if let Some(responses) = batch_request_output.responses.clone() {
            for item in responses.get(&self.table_name).unwrap_or(&Vec::new()) {
                let token_data: Result<TokenData, _> = item.clone().try_into();
                if let Ok(token_data) = token_data {
                    let mut contract_name = "".to_string();
                    if let Some(collection_responses) = &collections_request_output.responses {
                        for collection in collection_responses
                            .get(&self.table_name)
                            .unwrap_or(&Vec::new())
                        {
                            if let Some(AttributeValue::S(pk_value)) = collection.get("PK") {
                                if pk_value == &format!("CONTRACT#{}", token_data.contract_address)
                                {
                                    if let Some(AttributeValue::M(data_map)) =
                                        collection.get("Data")
                                    {
                                        if let Some(AttributeValue::S(name)) = data_map.get("Name")
                                        {
                                            contract_name.clone_from(name);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    token_results.push(BatchTokenData {
                        awaiting_metadata_update: token_data.awaiting_metadata_update,
                        contract_address: token_data.contract_address,
                        contract_name,
                        owner: token_data.owner,
                        token_id: token_data.token_id,
                        token_id_hex: token_data.token_id_hex,
                        metadata: token_data.metadata,
                        mint_info: token_data.mint_info,
                    });
                }
            }
        }

        let consumed_capacity_units = batch_request_output.consumed_capacity();

        let mut total_capacity_units: f64 = 0.0;
        consumed_capacity_units.iter().for_each(|c| {
            total_capacity_units += c.capacity_units().unwrap_or(0.0);
        });

        Ok(DynamoDbOutput::new(
            token_results,
            Some(total_capacity_units),
            None,
        ))
    }

    async fn get_contract_tokens(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        tokens_ids: &[String],
    ) -> Result<DynamoDbOutput<Vec<TokenData>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":contract".to_string(),
            AttributeValue::S(format!("CONTRACT#{}", contract_address)),
        );
        values.insert(
            ":token".to_string(),
            AttributeValue::S("TOKEN#".to_string()),
        );

        let filter_expression = if tokens_ids.is_empty() {
            None
        } else {
            let mut s = "GSI2SK IN (".to_string();

            for (idx, id) in tokens_ids.iter().enumerate() {
                let token_val = format!(":token{}", idx);

                values.insert(
                    token_val.clone(),
                    AttributeValue::S(format!("TOKEN#{}#{}", contract_address, id.clone())),
                );

                if idx < tokens_ids.len() - 1 {
                    s.push_str(&format!("{},", token_val));
                } else {
                    s.push_str(&format!("{})", token_val));
                }
            }

            Some(s)
        };

        let r = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI1PK-GSI1SK-index")
            .set_key_condition_expression(Some(
                "GSI1PK = :contract AND begins_with(GSI1SK, :token)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .set_filter_expression(filter_expression)
            .set_limit(self.limit)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = r.clone().items {
            for i in items {
                res.push(i.try_into()?);
            }
        }

        let consumed_capacity_units = match r.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        Ok(DynamoDbOutput::new_lek(
            res,
            consumed_capacity_units,
            r.last_evaluated_key,
            None,
        ))
    }

    async fn get_owner_all(
        &self,
        ctx: &DynamoDbCtx,
        owner_address: &str,
    ) -> Result<DynamoDbOutput<Vec<ContractWithTokens>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":owner".to_string(),
            AttributeValue::S(format!("OWNER#{}", owner_address)),
        );
        values.insert(
            ":token".to_string(),
            AttributeValue::S("TOKEN#".to_string()),
        );

        let mut items_by_contract: HashMap<String, Vec<BasicTokenData>> = HashMap::new();
        let mut last_evaluated_key = None;
        let mut consumed_capacity_units: f64 = 0.0;

        loop {
            let tokens_query_output = ctx
                .client
                .query()
                .table_name(&self.table_name)
                .index_name("GSI2PK-GSI2SK-index")
                .key_condition_expression("GSI2PK = :owner AND begins_with(GSI2SK, :token)")
                .set_expression_attribute_values(Some(values.clone()))
                .set_exclusive_start_key(last_evaluated_key)
                .return_consumed_capacity(ReturnConsumedCapacity::Total)
                .send()
                .await
                .map_err(|e| ProviderError::DatabaseError(format!("Query failed: {:?}", e)))?;

            if let Some(consumed_capacity) = tokens_query_output.consumed_capacity {
                consumed_capacity_units += consumed_capacity.capacity_units.unwrap_or(0.0);
            }

            for item in tokens_query_output.items.unwrap_or_default() {
                let token_data: TokenData = item.try_into().map_err(|e| {
                    ProviderError::SerializationError(format!("Deserialization failed: {:?}", e))
                })?;
                let basic_token_data = BasicTokenData {
                    awaiting_metadata_update: token_data.awaiting_metadata_update,
                    owner: token_data.owner,
                    token_id: token_data.token_id,
                    metadata: token_data.metadata,
                    mint_info: token_data.mint_info,
                };

                items_by_contract
                    .entry(token_data.contract_address.clone())
                    .or_default()
                    .push(basic_token_data);
            }

            last_evaluated_key = tokens_query_output.last_evaluated_key;
            if last_evaluated_key.is_none() {
                break;
            }
        }

        let mut results: Vec<ContractWithTokens> = Vec::new();
        for (contract_address, tokens) in items_by_contract {
            let contract_request_output = ctx
                .client
                .get_item()
                .table_name(&self.table_name)
                .key(
                    "PK",
                    AttributeValue::S(format!("CONTRACT#{}", &contract_address)),
                )
                .key("SK", AttributeValue::S("CONTRACT".to_string()))
                .return_consumed_capacity(ReturnConsumedCapacity::Total)
                .send()
                .await
                .map_err(|e| ProviderError::DatabaseError(format!("Get item failed: {:?}", e)))?;

            if let Some(consumed_capacity) = contract_request_output.consumed_capacity {
                consumed_capacity_units += consumed_capacity.capacity_units.unwrap_or(0.0);
            }

            let (contract_name, contract_symbol, contract_image) =
                match contract_request_output.item {
                    Some(contract_item) => match contract_item.get("Data") {
                        Some(data_av) => {
                            let mut result_name: Option<String> = None;
                            let mut result_symbol: Option<String> = None;
                            let mut result_image: Option<String> = None;

                            let data = data_av.as_m();
                            if data.is_ok() {
                                if let Some(name) =
                                    data.unwrap().get("Name").and_then(|n| n.as_s().ok())
                                {
                                    result_name = Some(name.to_owned());
                                }

                                if let Some(symbol) =
                                    data.unwrap().get("Symbol").and_then(|s| s.as_s().ok())
                                {
                                    result_symbol = Some(symbol.to_owned());
                                }

                                if let Some(image) =
                                    data.unwrap().get("Image").and_then(|s| s.as_s().ok())
                                {
                                    result_image = Some(image.to_owned());
                                }
                            }

                            (result_name, result_symbol, result_image)
                        }
                        None => (None, None, None),
                    },
                    None => (None, None, None),
                };

            results.push(ContractWithTokens {
                contract_address,
                contract_name,
                contract_symbol,
                contract_image,
                items: tokens,
            });
        }

        Ok(DynamoDbOutput::new(
            results.clone(),
            Some(consumed_capacity_units),
            Some(results.clone().len() as i32),
        ))
    }

    async fn get_owner_contracts_addresses(
        &self,
        ctx: &DynamoDbCtx,
        owner_address: &str,
    ) -> Result<DynamoDbOutput<Vec<ContractData>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":owner".to_string(),
            AttributeValue::S(format!("OWNER#{}", owner_address)),
        );
        values.insert(
            ":token".to_string(),
            AttributeValue::S("TOKEN#".to_string()),
        );

        let mut total_by_contract: HashMap<String, u64> = HashMap::new();
        let mut contracts: Vec<String> = Vec::new();
        let mut last_evaluated_key = ctx.exclusive_start_key.clone();
        let mut consumed_capacity_units: f64 = 0.0;

        loop {
            let query_output = ctx
                .client
                .query()
                .table_name(&self.table_name)
                .index_name("GSI2PK-GSI2SK-index")
                .set_key_condition_expression(Some(
                    "GSI2PK = :owner AND begins_with(GSI2SK, :token)".to_string(),
                ))
                .set_expression_attribute_values(Some(values.clone()))
                .set_exclusive_start_key(last_evaluated_key)
                .return_consumed_capacity(ReturnConsumedCapacity::Total)
                .send()
                .await
                .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

            if let Some(consumed_capacity) = query_output.consumed_capacity {
                if let Some(capacity_units) = consumed_capacity.capacity_units {
                    consumed_capacity_units += capacity_units;
                }
            }

            if let Some(items) = query_output.items {
                for item in items {
                    let token_data: TokenData = item.try_into()?;

                    if total_by_contract.contains_key(&token_data.contract_address) {
                        let count = total_by_contract
                            .get_mut(&token_data.contract_address)
                            .unwrap();
                        *count += 1;
                    } else {
                        total_by_contract.insert(token_data.contract_address.clone(), 1);
                    }

                    if !contracts.contains(&token_data.contract_address) {
                        contracts.push(token_data.contract_address)
                    }
                }
            }

            if query_output.last_evaluated_key.is_none() {
                break;
            }
            last_evaluated_key = query_output.last_evaluated_key;
        }

        let mut contracts_list: Vec<ContractData> = Vec::new();
        for contract in contracts {
            let tokens_count = match total_by_contract.get(&contract) {
                Some(t) => *t,
                None => 0,
            };

            contracts_list.push(ContractData {
                contract_address: contract,
                tokens_count,
            });
        }

        let total_count = contracts_list.len() as i32;

        Ok(DynamoDbOutput::new(
            contracts_list,
            Some(consumed_capacity_units),
            Some(total_count),
        ))
    }

    async fn get_owner_tokens(
        &self,
        ctx: &DynamoDbCtx,
        owner_address: &str,
        contract_address: Option<String>,
    ) -> Result<DynamoDbOutput<Vec<TokenData>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":owner".to_string(),
            AttributeValue::S(format!("OWNER#{}", owner_address)),
        );

        let token_prefix = if let Some(contract_address) = contract_address {
            format!("TOKEN#{}", contract_address)
        } else {
            "TOKEN#".to_string()
        };

        values.insert(":token".to_string(), AttributeValue::S(token_prefix));

        // Requête pour obtenir le nombre total d'éléments
        let count_query_output = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some(
                "GSI2PK = :owner AND begins_with(GSI2SK, :token)".to_string(),
            ))
            .set_expression_attribute_values(Some(values.clone()))
            .select(Select::Count)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let total_count = count_query_output.count;

        let query_output = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some(
                "GSI2PK = :owner AND begins_with(GSI2SK, :token)".to_string(),
            ))
            .set_expression_attribute_values(Some(values.clone()))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = Vec::new();
        if let Some(items) = query_output.clone().items {
            for item in items {
                res.push(item.try_into()?);
            }
        }

        let consumed_capacity_units = match query_output.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        let output = DynamoDbOutput::new_lek(
            res,
            consumed_capacity_units,
            query_output.last_evaluated_key,
            Some(total_count),
        );

        Ok(output)
    }
}

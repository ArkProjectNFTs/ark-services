use crate::providers::token::types::TokenData;
use crate::providers::ArkTokenProvider;
use crate::{convert, DynamoDbCtx, DynamoDbOutput, EntityType, ProviderError};

use arkproject::metadata::types::TokenMetadata;
use arkproject::pontos::storage::types::TokenMintInfo;
use arkproject::starknet::CairoU256;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnConsumedCapacity};
use aws_sdk_dynamodb::Client as DynamoClient;
use chrono::Utc;
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use tracing::{debug, info, trace};

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
            "Updating token metadata status for contract address: {}, token_id: {}",
            contract_address,
            token_id_hex
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
        info!("get_token: pk={}", pk);
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
        info!("get_token: pk={}", pk);
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
            let data = convert::attr_to_map(item, "Data")?;
            let token_data: TokenData = data.try_into()?;
            Ok(DynamoDbOutput::new(Some(token_data), &r.consumed_capacity))
        } else {
            Ok(DynamoDbOutput::new(None, &r.consumed_capacity))
        }
    }

    async fn get_token_without_metadata(
        &self,
        client: &Self::Client,
        contract_address_filter: Option<FieldElement>,
    ) -> Result<Vec<(FieldElement, CairoU256)>, ProviderError> {
        let sort_key = match contract_address_filter {
            Some(contract_address) => format!("CONTRACT#0x{:064x}", contract_address),
            None => "CONTRACT".to_string(),
        };
        let query_result = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI5PK-GSI5SK-index")
            .key_condition_expression("GSI5PK = :gsi_pk AND begins_with(GSI5SK, :gsi_sk)")
            .expression_attribute_values(
                ":gsi_pk",
                AttributeValue::S(String::from("METADATA#false")),
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

                let mut results: Vec<(FieldElement, CairoU256)> = Vec::new();
                let items = query_output.items.unwrap();

                for item in items.iter() {
                    if let Some(data) = item.get("Data") {
                        if data.is_m() {
                            let data_m = data.as_m().unwrap();

                            // Extracting contract address
                            match data_m.get("ContractAddress") {
                                Some(AttributeValue::S(contract_address_attribute_value)) => {
                                    let contract_address_result =
                                        FieldElement::from_hex_be(contract_address_attribute_value);

                                    let contract_address = match contract_address_result {
                                        Ok(address) => address,
                                        Err(_) => {
                                            return Err(ProviderError::ParsingError(format!(
                                                "Failed to parse contract address from: {}",
                                                contract_address_attribute_value
                                            )));
                                        }
                                    };

                                    // Extracting token ID
                                    if let Some(AttributeValue::S(token_id_attribute_value)) =
                                        data_m.get("TokenIdHex")
                                    {
                                        let cairo_u256_result =
                                            CairoU256::from_hex_be(token_id_attribute_value);

                                        match cairo_u256_result {
                                            Ok(token_id) => {
                                                results.push((contract_address, token_id));
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
                AttributeValue::S("METADATA#false".to_string()),
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
        if let Some(items) = r.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(data.try_into()?);
            }
        }

        Ok(DynamoDbOutput::new_lek(
            res,
            &r.consumed_capacity,
            r.last_evaluated_key,
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

        if let Some(contract_address) = contract_address {
            values.insert(
                ":token".to_string(),
                AttributeValue::S(format!("TOKEN#{}", contract_address)),
            );
        } else {
            values.insert(
                ":token".to_string(),
                AttributeValue::S("TOKEN#".to_string()),
            );
        }

        let r = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some(
                "GSI2PK = :owner AND begins_with(GSI2SK, :token)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = r.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(data.try_into()?);
            }
        }

        Ok(DynamoDbOutput::new_lek(
            res,
            &r.consumed_capacity,
            r.last_evaluated_key,
        ))
    }
}

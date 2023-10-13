use arkproject::metadata::types::TokenMetadata;
use arkproject::pontos::storage::types::TokenMintInfo;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnConsumedCapacity};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

use super::ArkTokenProvider;
use crate::providers::metrics::DynamoDbCapacityProvider;
use crate::providers::token::types::TokenData;
use crate::{convert, EntityType, ProviderError, DynamoDbOutput, DynamoDbCtx};

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
        let sk = self.get_sk();

        let r = ctx.client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            .update_expression("SET #data.#owner = :owner")
            .expression_attribute_names("#data", "Data")
            .expression_attribute_names("#owner", "Owner")
            .expression_attribute_values(":owner".to_string(), AttributeValue::S(owner.to_string()))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            &ctx.client,
            "update_owner",
            &r.consumed_capacity,
        )
        .await;

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

        let r = ctx.client
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
            .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            &ctx.client,
            "update_mint_info",
            &r.consumed_capacity,
        )
        .await;

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
        let data = TokenData::metadata_to_map(metadata);

        let mut names = HashMap::new();
        names.insert("#data".to_string(), "Data".to_string());
        names.insert("#metadata".to_string(), "Metadata".to_string());

        let r = ctx.client
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
            .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            &ctx.client,
            "update_metadata",
            &r.consumed_capacity,
        )
        .await;

        Ok(().into())
    }

    async fn get_token(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
    ) -> Result<DynamoDbOutput<Option<TokenData>>, ProviderError> {
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(contract_address, token_id_hex)),
        );
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let r = ctx.client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            &ctx.client,
            "get_token",
            &r.consumed_capacity,
        )
        .await;

        if let Some(item) = &r.item {
            let data = convert::attr_to_map(item, "Data")?;
            let token_data: TokenData = data.try_into()?;
            Ok(DynamoDbOutput::new(Some(token_data), &r.consumed_capacity))
        } else {
            Ok(DynamoDbOutput::new(None, &r.consumed_capacity))
        }
    }

    async fn register_token(
        &self,
        ctx: &DynamoDbCtx,
        info: &TokenData,
        block_timestamp: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let pk = self.get_pk(&info.contract_address, &info.token_id_hex);

        let r = ctx.client
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
            .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            &ctx.client,
            "register_token",
            &r.consumed_capacity,
        )
        .await;

        Ok(().into())
    }

    async fn get_contract_tokens(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
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

        let r = ctx.client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI1PK-GSI1SK-index")
            .set_key_condition_expression(Some(
                "GSI1PK = :contract AND begins_with(GSI1SK, :token)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .set_limit(self.limit)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            &ctx.client,
            "get_contract_tokens",
            &r.consumed_capacity,
        )
        .await;

        let mut res = vec![];
        if let Some(items) = r.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(data.try_into()?);
            }
        }

        Ok(DynamoDbOutput::new_lek(res, &r.consumed_capacity, r.last_evaluated_key))
    }

    async fn get_owner_tokens(
        &self,
        ctx: &DynamoDbCtx,
        owner_address: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenData>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":owner".to_string(),
            AttributeValue::S(format!("OWNER#{}", owner_address)),
        );
        values.insert(
            ":token".to_string(),
            AttributeValue::S("TOKEN#".to_string()),
        );

        let r = ctx.client
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

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            &ctx.client,
            "get_owner_tokens",
            &r.consumed_capacity,
        )
        .await;

        let mut res = vec![];
        if let Some(items) = r.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(data.try_into()?);
            }
        }

        Ok(DynamoDbOutput::new_lek(res, &r.consumed_capacity, r.last_evaluated_key))
    }
}

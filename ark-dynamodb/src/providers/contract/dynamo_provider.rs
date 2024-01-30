use arkproject::pontos::storage::types::ContractInfo;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, ReturnConsumedCapacity};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use tracing::{debug, trace};

use super::ArkContractProvider;
use crate::{convert, DynamoDbCtx, DynamoDbOutput, EntityType, ProviderError};

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

        let r = ctx
            .client
            .batch_get_item()
            .request_items(
                self.table_name.clone(),
                KeysAndAttributes::builder().set_keys(Some(keys)).build(),
            )
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut contract_infos = Vec::new();
        if let Some(responses) = r.responses {
            for item in responses.get(&self.table_name).unwrap_or(&Vec::new()) {
                let data = convert::attr_to_map(item, "Data")?;
                contract_infos.push(Self::data_to_info(&data)?);
            }
        }

        Ok(DynamoDbOutput::new(contract_infos, &r.consumed_capacity))
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
            Ok(DynamoDbOutput::new(
                Some(Self::data_to_info(&data)?),
                &r.consumed_capacity,
            ))
        } else {
            Ok(DynamoDbOutput::new(None, &r.consumed_capacity))
        }
    }

    async fn get_nft_contracts(
        &self,
        ctx: &DynamoDbCtx,
    ) -> Result<DynamoDbOutput<Vec<ContractInfo>>, ProviderError> {
        trace!("get_nft_contracts");

        let mut values = HashMap::new();
        values.insert(":pk".to_string(), AttributeValue::S("NFT".to_string()));

        let r = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some("GSI2PK = :pk".to_string()))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .set_limit(self.limit)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = r.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(Self::data_to_info(&data)?);
            }
        }

        Ok(DynamoDbOutput::new_lek(
            res,
            &r.consumed_capacity,
            r.last_evaluated_key,
            None,
        ))
    }
}

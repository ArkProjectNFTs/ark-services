use arkproject::pontos::storage::types::ContractInfo;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

use super::ArkContractProvider;
use crate::{convert, EntityType, ProviderError};

/// DynamoDB provider for contracts.
pub struct DynamoDbContractProvider {
    table_name: String,
    key_prefix: String,
}

impl DynamoDbContractProvider {
    pub fn new(table_name: &str) -> Self {
        DynamoDbContractProvider {
            table_name: table_name.to_string(),
            key_prefix: "CONTRACT".to_string(),
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
        Ok(ContractInfo {
            contract_type: convert::attr_to_str(data, "ContractType")?,
            contract_address: convert::attr_to_str(data, "ContractAddress")?,
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
        client: &Self::Client,
        info: &ContractInfo,
        block_number: u64,
    ) -> Result<(), ProviderError> {
        let pk = self.get_pk(&info.contract_address);
        let sk = self.get_sk();

        let data = Self::info_to_data(info);

        let put_item_output = client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(pk.clone()))
            .item("SK", AttributeValue::S(sk.clone()))
            // We can't filter on PK/SK to only get all contract, as the PK
            // is required. So we duplicate info in the GSI1. TODO: investiagte more.
            .item("GSI1PK".to_string(), AttributeValue::S(sk.clone()))
            .item("GSI1SK".to_string(), AttributeValue::S(pk.clone()))
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_number)),
            )
            .item("GSI4SK".to_string(), AttributeValue::S(pk.clone()))
            .item("Data", AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Contract.to_string()))
            .condition_expression("attribute_not_exists(PK)")
            .send()
            .await;

        put_item_output.map_err(|e| ProviderError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_contract(
        &self,
        client: &Self::Client,
        contract_address: &str,
    ) -> Result<Option<ContractInfo>, ProviderError> {
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(contract_address)),
        );
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let req = client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        if let Some(item) = &req.item {
            let data = convert::attr_to_map(item, "Data")?;
            Ok(Some(Self::data_to_info(&data)?))
        } else {
            Ok(None)
        }
    }

    async fn get_contracts(
        &self,
        client: &Self::Client,
    ) -> Result<Vec<ContractInfo>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":contract".to_string(),
            AttributeValue::S("CONTRACT".to_string()),
        );

        let req = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI1PK-GSI1SK-index")
            .set_key_condition_expression(Some(
                "GSI1PK = :contract AND begins_with(GSI1SK, :contract)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = req.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(Self::data_to_info(&data)?);
            }
        }

        Ok(res)
    }
}

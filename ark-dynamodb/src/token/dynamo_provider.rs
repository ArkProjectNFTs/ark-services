use arkproject::pontos::storage::types::TokenInfo;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

use super::ArkTokenProvider;
use crate::{convert, EntityType, ProviderError};

/// DynamoDB provider for tokens.
pub struct DynamoDbTokenProvider {
    table_name: String,
    key_prefix: String,
}

impl DynamoDbTokenProvider {
    pub fn new(table_name: &str) -> Self {
        DynamoDbTokenProvider {
            table_name: table_name.to_string(),
            key_prefix: "TOKEN".to_string(),
        }
    }

    fn get_pk(&self, contract_address: &str, token_id: &str) -> String {
        format!("{}#{}#{}", self.key_prefix, contract_address, token_id)
    }

    fn get_sk(&self) -> String {
        self.key_prefix.clone()
    }

    pub fn data_to_info(
        data: &HashMap<String, AttributeValue>,
    ) -> Result<TokenInfo, ProviderError> {
        let mint_block_number = match convert::attr_to_u64(data, "MintBlockNumber") {
            Ok(v) => Some(v),
            _ => None,
        };
        let mint_timestamp = match convert::attr_to_u64(data, "MintTimestamp") {
            Ok(v) => Some(v),
            _ => None,
        };
        let mint_transaction_hash = match convert::attr_to_str(data, "MintTransactionHash") {
            Ok(v) => Some(v),
            _ => None,
        };
        let mint_address = match convert::attr_to_str(data, "MintAddress") {
            Ok(v) => Some(v),
            _ => None,
        };

        Ok(TokenInfo {
            owner: convert::attr_to_str(data, "Owner")?,
            address: convert::attr_to_str(data, "ContractAddress")?,
            token_id: convert::attr_to_str(data, "TokenId")?,
            token_id_hex: convert::attr_to_str(data, "TokenIdHex")?,
            mint_block_number,
            mint_timestamp,
            mint_transaction_hash,
            mint_address,
        })
    }

    pub fn info_to_data(info: &TokenInfo) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert("Owner".to_string(), AttributeValue::S(info.owner.clone()));
        map.insert(
            "ContractAddress".to_string(),
            AttributeValue::S(info.address.clone()),
        );
        map.insert(
            "TokenId".to_string(),
            AttributeValue::S(info.token_id.clone()),
        );
        map.insert(
            "TokenIdHex".to_string(),
            AttributeValue::S(info.token_id_hex.clone()),
        );

        if let Some(v) = info.mint_block_number {
            map.insert(
                "MintBlockNumber".to_string(),
                AttributeValue::N(v.to_string()),
            );
        }
        if let Some(v) = info.mint_timestamp {
            map.insert(
                "MintTimestamp".to_string(),
                AttributeValue::N(v.to_string()),
            );
        }
        if let Some(v) = &info.mint_address {
            map.insert("MintAddress".to_string(), AttributeValue::S(v.clone()));
        }
        if let Some(v) = &info.mint_transaction_hash {
            map.insert(
                "MintTransactionHash".to_string(),
                AttributeValue::S(v.clone()),
            );
        }

        map
    }
}

#[async_trait]
impl ArkTokenProvider for DynamoDbTokenProvider {
    type Client = DynamoClient;

    async fn update_data(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_id_hex: &str,
        data: HashMap<String, AttributeValue>,
    ) -> Result<(), ProviderError> {
        let pk = self.get_pk(contract_address, token_id_hex);
        let sk = self.get_sk();

        let test = {
            let mut values = HashMap::new();
            values.insert(
                ":new_name".to_string(),
                AttributeValue::S("doe".to_string()),
            );
            values.insert(
                ":new_city".to_string(),
                AttributeValue::S("ici".to_string()),
            );
            values
        };

        let update_item_output = client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            //.update_expression("SET Data = :data")
            .update_expression("SET Data.name = :new_name, Data.city = :new_city")
            //.expression_attribute_values(":data".to_string(), AttributeValue::M(data))
            .expression_attribute_values(":Data".to_string(), AttributeValue::M(test))
            .return_values(ReturnValue::AllNew)
            .send()
            .await;

        update_item_output.map_err(|e| ProviderError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_token(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_id_hex: &str,
    ) -> Result<Option<TokenInfo>, ProviderError> {
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(contract_address, token_id_hex)),
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

    async fn register_token(
        &self,
        client: &Self::Client,
        info: &TokenInfo,
        block_number: u64,
    ) -> Result<(), ProviderError> {
        let data = Self::info_to_data(info);

        let pk = self.get_pk(&info.address, &info.token_id_hex);

        let put_item_output = client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK".to_string(), AttributeValue::S(pk.clone()))
            .item("SK".to_string(), AttributeValue::S(self.get_sk()))
            .item("Type".to_string(), AttributeValue::S("Token".to_string()))
            .item(
                "GSI1PK".to_string(),
                AttributeValue::S(format!("COLLECTION#{}", info.address)),
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
                AttributeValue::S(format!("BLOCK#{}", block_number)),
            )
            .item("GSI4SK".to_string(), AttributeValue::S(pk.clone()))
            .item("Data".to_string(), AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Token.to_string()))
            .return_values(ReturnValue::AllOld)
            .send()
            .await;

        put_item_output.map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

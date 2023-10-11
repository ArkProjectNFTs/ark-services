use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

use super::types::TokenData;
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
}

#[async_trait]
impl ArkTokenProvider for DynamoDbTokenProvider {
    type Client = DynamoClient;

    async fn update_owner(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_id_hex: &str,
        owner: &str,
    ) -> Result<(), ProviderError> {
        let pk = self.get_pk(contract_address, token_id_hex);
        let sk = self.get_sk();

        let update_item_output = client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            .update_expression("SET #data.#owner = :owner")
            .expression_attribute_names("#data", "Data")
            .expression_attribute_names("#owner", "Owner")
            .expression_attribute_values(":owner".to_string(), AttributeValue::S(owner.to_string()))
            .return_values(ReturnValue::AllNew)
            .send()
            .await;

        update_item_output.map_err(|e| ProviderError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_mint_data(
        &self,
        client: &Self::Client,
        info: &TokenData,
    ) -> Result<(), ProviderError> {
        let pk = self.get_pk(&info.contract_address, &info.token_id_hex);
        let sk = self.get_sk();

        let mut values = HashMap::new();
        values.insert(
            ":addr".to_string(),
            AttributeValue::S(info.mint_address.clone().unwrap_or_default()),
        );
        values.insert(
            ":tx".to_string(),
            AttributeValue::S(info.mint_transaction_hash.clone().unwrap_or_default()),
        );
        values.insert(
            ":ts".to_string(),
            AttributeValue::N(info.mint_timestamp.unwrap_or(0).to_string()),
        );
        values.insert(
            ":bn".to_string(),
            AttributeValue::N(info.mint_block_number.unwrap_or(0).to_string()),
        );

        let mut names = HashMap::new();
        names.insert("#data".to_string(), "Data".to_string());
        names.insert("#addr".to_string(), "MintAddress".to_string());
        names.insert("#tx".to_string(), "MintTransactionHash".to_string());
        names.insert("#ts".to_string(), "MintTimestamp".to_string());
        names.insert("#bn".to_string(), "MintBlockNumber".to_string());

        let update_item_output = client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            .update_expression(
                "SET #data.#addr = :addr, #data.#tx = :tx, #data.#ts = :ts, #data.#bn = :bn",
            )
            .set_expression_attribute_names(Some(names))
            .set_expression_attribute_values(Some(values))
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
    ) -> Result<Option<TokenData>, ProviderError> {
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
            let mut data = convert::attr_to_map(item, "Data")?;
            let result: TokenData = data.try_into()?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn register_token(
        &self,
        client: &Self::Client,
        token_data: &TokenData,
        block_number: u64,
    ) -> Result<(), ProviderError> {
        let data: HashMap<String, AttributeValue> = token_data.into();

        // let data = Self::info_to_data(info);

        let pk = self.get_pk(&token_data.contract_address, &token_data.token_id_hex);

        let put_item_output = client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK".to_string(), AttributeValue::S(pk.clone()))
            .item("SK".to_string(), AttributeValue::S(self.get_sk()))
            .item("Type".to_string(), AttributeValue::S("Token".to_string()))
            .item(
                "GSI1PK".to_string(),
                AttributeValue::S(format!("CONTRACT#{}", token_data.contract_address)),
            )
            .item(
                "GSI1SK".to_string(),
                AttributeValue::S(format!("TOKEN#{}", token_data.token_id_hex)),
            )
            .item(
                "GSI2PK".to_string(),
                AttributeValue::S(format!("OWNER#{}", token_data.owner)),
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

    async fn get_contract_tokens(
        &self,
        client: &Self::Client,
        contract_address: &str,
    ) -> Result<Vec<TokenData>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":contract".to_string(),
            AttributeValue::S(format!("CONTRACT#{}", contract_address)),
        );
        values.insert(
            ":token".to_string(),
            AttributeValue::S("TOKEN#".to_string()),
        );

        let req = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI1PK-GSI1SK-index")
            .set_key_condition_expression(Some(
                "GSI1PK = :contract AND begins_with(GSI1SK, :token)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = req.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(data.try_into()?);
            }
        }

        Ok(res)
    }

    async fn get_owner_tokens(
        &self,
        client: &Self::Client,
        owner_address: &str,
    ) -> Result<Vec<TokenData>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(
            ":owner".to_string(),
            AttributeValue::S(format!("OWNER#{}", owner_address)),
        );
        values.insert(
            ":token".to_string(),
            AttributeValue::S("TOKEN#".to_string()),
        );

        let req = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some(
                "GSI2PK = :owner AND begins_with(GSI2SK, :token)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = req.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(data.try_into()?);
            }
        }

        Ok(res)
    }
}

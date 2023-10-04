use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

use super::{types::CollectionData, ArkCollectionProvider};
use crate::{convert, ProviderError};

/// DynamoDB provider for collections.
pub struct DynamoDbCollectionProvider {
    table_name: String,
    key_prefix: String,
}

impl DynamoDbCollectionProvider {
    pub fn new(table_name: &str) -> Self {
        DynamoDbCollectionProvider {
            table_name: table_name.to_string(),
            key_prefix: "COLLECTION".to_string(),
        }
    }

    fn get_pk(&self, contract_address: &str) -> String {
        format!("{}#{}", self.key_prefix, contract_address)
    }
}

#[async_trait]
impl ArkCollectionProvider for DynamoDbCollectionProvider {
    type Client = DynamoClient;

    async fn get_collection(
        &self,
        client: &Self::Client,
        contract_address: &str,
    ) -> Result<Option<CollectionData>, ProviderError> {
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
            Ok(Some(data.try_into()?))
        } else {
            Ok(None)
        }
    }
}

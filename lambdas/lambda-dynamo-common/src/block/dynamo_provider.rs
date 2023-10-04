use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

use super::{types::BlockData, ArkBlockProvider};
use crate::{convert, ProviderError};

/// DynamoDB provider for blocks.
pub struct DynamoDbBlockProvider {
    table_name: String,
    key_prefix: String,
}

impl DynamoDbBlockProvider {
    pub fn new(table_name: &str) -> Self {
        DynamoDbBlockProvider {
            table_name: table_name.to_string(),
            key_prefix: "BLOCK".to_string(),
        }
    }

    fn get_pk(&self, block_number: u64) -> String {
        format!("{}#{}", self.key_prefix, block_number)
    }
}

#[async_trait]
impl ArkBlockProvider for DynamoDbBlockProvider {
    type Client = DynamoClient;

    async fn get_block(
        &self,
        client: &Self::Client,
        block_number: u64,
    ) -> Result<Option<BlockData>, ProviderError> {
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(block_number)),
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

            Ok(Some(BlockData {
                status: convert::attr_to_str(&data, "Status")?,
                indexer_identifier: convert::attr_to_str(&data, "IndexerIdentifier")?,
                indexer_version: convert::attr_to_str(&data, "IndexerVersion")?,
            }))
        } else {
            Ok(None)
        }
    }
}

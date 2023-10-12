use arkproject::pontos::storage::types::{BlockIndexingStatus, BlockInfo};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnConsumedCapacity};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::str::FromStr;

use super::ArkBlockProvider;
use crate::providers::DynamoDbCapacityProvider;
use crate::{convert, EntityType, ProviderError};

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

    fn get_sk(&self) -> String {
        self.key_prefix.to_string()
    }

    pub fn data_to_info(
        data: &HashMap<String, AttributeValue>,
    ) -> Result<BlockInfo, ProviderError> {
        Ok(BlockInfo {
            status: BlockIndexingStatus::from_str(&convert::attr_to_str(data, "Status")?).map_err(
                |_| ProviderError::DataValueError("BlockIndexingStatus parse failed".to_string()),
            )?,
            indexer_identifier: convert::attr_to_str(data, "IndexerIdentifier")?,
            indexer_version: convert::attr_to_str(data, "IndexerVersion")?,
        })
    }

    pub fn info_to_data(data: &BlockInfo) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            "IndexerVersion".to_string(),
            AttributeValue::S(data.indexer_version.clone()),
        );
        map.insert(
            "IndexerIdentifier".to_string(),
            AttributeValue::S(data.indexer_identifier.clone()),
        );
        map.insert(
            "Status".to_string(),
            AttributeValue::S(data.status.to_string()),
        );

        map
    }
}

#[async_trait]
impl ArkBlockProvider for DynamoDbBlockProvider {
    type Client = DynamoClient;

    async fn set_info(
        &self,
        client: &Self::Client,
        block_number: u64,
        block_timestamp: u64,
        info: &BlockInfo,
    ) -> Result<(), ProviderError> {
        let data = DynamoDbBlockProvider::info_to_data(info);

        let r = client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(self.get_pk(block_number)))
            .item("SK", AttributeValue::S(self.get_sk()))
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(self.get_pk(block_timestamp)),
            )
            .item("GSI4SK".to_string(), AttributeValue::S(self.get_sk()))
            .item("Data", AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Block.to_string()))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            client,
            "block_set_info",
            r.consumed_capacity,
        )
        .await;

        Ok(())
    }

    async fn get_info(
        &self,
        client: &Self::Client,
        block_number: u64,
    ) -> Result<Option<BlockInfo>, ProviderError> {
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(block_number)),
        );
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let r = client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let _ = DynamoDbCapacityProvider::register_consumed_capacity(
            client,
            "block_get_info",
            r.consumed_capacity,
        )
        .await;

        if let Some(item) = &r.item {
            let data = convert::attr_to_map(item, "Data")?;
            Ok(Some(DynamoDbBlockProvider::data_to_info(&data)?))
        } else {
            Ok(None)
        }
    }
}

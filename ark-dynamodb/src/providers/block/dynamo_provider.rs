use arkproject::pontos::storage::types::{BlockIndexingStatus, BlockInfo};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    AttributeValue, DeleteRequest, ReturnConsumedCapacity, WriteRequest,
};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::trace;

use super::ArkBlockProvider;
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
            block_number: convert::attr_to_u64(data, "BlockNumber").unwrap_or(0),
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
        map.insert(
            "BlockNumber".to_string(),
            AttributeValue::N(data.block_number.to_string()),
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

        let _r = client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(self.get_pk(block_number)))
            .item("SK", AttributeValue::S(self.get_sk()))
            .item(
                "GSI1PK".to_string(),
                AttributeValue::S(String::from("BLOCK")),
            )
            .item(
                "GSI1SK".to_string(),
                AttributeValue::S(self.get_pk(block_timestamp)),
            )
            .item("Data", AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Block.to_string()))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        Ok(())
    }

    async fn get_info(
        &self,
        client: &Self::Client,
        block_number: u64,
    ) -> Result<Option<BlockInfo>, ProviderError> {
        trace!("get_info: block_number: {}", block_number);

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

        if let Some(item) = &r.item {
            let data = convert::attr_to_map(item, "Data")?;
            Ok(Some(DynamoDbBlockProvider::data_to_info(&data)?))
        } else {
            Ok(None)
        }
    }

    async fn clean(
        &self,
        client: &Self::Client,
        block_timestamp: u64,
        block_number: Option<u64>,
    ) -> Result<(), ProviderError> {
        let gsi_pk = format!("BLOCK#{}", block_timestamp);

        // let mut capacity: f64 = 0.0;

        // Query for all items associated with the block number
        let query_output = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI4PK-GSI4SK-index") // Assuming your GSI for block association is named GSI4
            .key_condition_expression("GSI4PK = :gsi_pk")
            .expression_attribute_values(":gsi_pk", AttributeValue::S(gsi_pk))
            .projection_expression("PK, SK") // Only retrieve necessary attributes
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("query error {:?}", e)))?;

        // if let Some(cc) = query_output.consumed_capacity {
        //     capacity += cc.capacity_units.unwrap_or(0.0);
        // }

        // Prepare the items for batch deletion
        let mut write_requests: Vec<WriteRequest> = Vec::new();
        if let Some(items) = query_output.items {
            for item in items {
                if let Some(pk) = item.get("PK").cloned() {
                    if let Some(sk) = item.get("SK").cloned() {
                        let delete_request =
                            DeleteRequest::builder().key("PK", pk).key("SK", sk).build();
                        let write_request = WriteRequest::builder()
                            .delete_request(delete_request)
                            .build();
                        write_requests.push(write_request);
                    }
                }
            }
        }

        // Batch delete items in chunks of 25
        for chunk in write_requests.chunks(25) {
            let batch_write_output = client
                .batch_write_item()
                .request_items(&self.table_name, chunk.to_vec())
                .return_consumed_capacity(ReturnConsumedCapacity::Total)
                .send()
                .await
                .map_err(|e| ProviderError::DatabaseError(format!("batch write error {:?}", e)))?;

            // if let Some(ccs) = batch_write_output.consumed_capacity {
            //     for cc in ccs {
            //         capacity += cc.capacity_units.unwrap_or(0.0);
            //     }
            // }

            // Handle unprocessed items if any
            if let Some(unprocessed_items) = batch_write_output.unprocessed_items {
                if let Some(retry_items) = unprocessed_items.get(&self.table_name) {
                    // Implement retry logic as per your use case
                    // Here, we'll simply wait for a second and try again
                    sleep(Duration::from_secs(1)).await;
                    let _o = client
                        .batch_write_item()
                        .request_items(&self.table_name, retry_items.clone())
                        .return_consumed_capacity(ReturnConsumedCapacity::Total)
                        .send()
                        .await
                        .map_err(|e| {
                            ProviderError::DatabaseError(format!("retry batch write error {:?}", e))
                        })?;

                    // if let Some(ccs) = o.consumed_capacity {
                    //     for cc in ccs {
                    //         capacity += cc.capacity_units.unwrap_or(0.0);
                    //     }
                    // }
                }
            }
        }

        // Delete the block entry only if we asked for.
        // TODO: I think that in our design, this is not required as the block entry has
        // also the GSI4PK set to the timestamp. So it will be deleted too.
        if let Some(block_number) = block_number {
            let pk_block = format!("BLOCK#{}", block_number);
            let sk_block = "BLOCK".to_string();
            let _o = client
                .delete_item()
                .table_name(&self.table_name)
                .key("PK", AttributeValue::S(pk_block))
                .key("SK", AttributeValue::S(sk_block))
                .return_consumed_capacity(ReturnConsumedCapacity::Total)
                .send()
                .await
                .map_err(|e| {
                    ProviderError::DatabaseError(format!("delete block entry error {:?}", e))
                })?;

            // if let Some(cc) = o.consumed_capacity {
            //     capacity += cc.capacity_units.unwrap_or(0.0);
            // }
        }

        Ok(())
    }
}

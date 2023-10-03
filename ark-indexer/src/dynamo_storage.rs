use anyhow::{anyhow, Result};
use arkproject::pontos::storage::{
    types::{
        BlockIndexingStatus, BlockInfo, ContractType, IndexerStatus, StorageError, TokenEvent,
        TokenFromEvent,
    },
    Storage,
};
use async_trait::async_trait;
use aws_config::load_from_env;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::Utc;
use log::{debug, error, trace};
use starknet::core::types::FieldElement;

pub struct DynamoStorage {
    client: Client,
    table_name: String,
}

impl DynamoStorage {
    pub async fn new(table_name: String) -> Self {
        let config = load_from_env().await;
        let client = Client::new(&config);
        Self { client, table_name }
    }
}

#[async_trait]
pub trait AWSDynamoStorage: Send + Sync {
    async fn update_indexer_task_status(
        &self,
        task_id: String,
        indexer_version: u64,
        status: IndexerStatus,
    ) -> Result<()>;
    async fn update_indexer_progress(&self, task_id: String, value: f64) -> Result<()>;
}

#[async_trait]
impl AWSDynamoStorage for DynamoStorage {
    async fn update_indexer_task_status(
        &self,
        task_id: String,
        indexer_version: u64,
        status: IndexerStatus,
    ) -> Result<()> {
        let now = Utc::now();
        let unix_timestamp = now.timestamp();

        let status_string = match status {
            IndexerStatus::Running => "running",
            IndexerStatus::Stopped => "stopped",
        }
        .to_string();

        let response = self
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(String::from("INDEXER")))
            .item("SK", AttributeValue::S(format!("TASK#{}", task_id)))
            .item("status", AttributeValue::S(status.to_string()))
            .item("last_update", AttributeValue::N(unix_timestamp.to_string()))
            .item("version", AttributeValue::N(indexer_version.to_string()))
            .item("task_id", AttributeValue::S(task_id.to_string()))
            .send()
            .await;

        match response {
            Ok(_) => {
                debug!("Successfully updated indexer task status for task_id {}: status {}, version {}", task_id, status_string, indexer_version);
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to update indexer task status for task_id {}: {:?}",
                    task_id, e
                );
                Err(e.into())
            }
        }
    }

    async fn update_indexer_progress(&self, task_id: String, value: f64) -> Result<()> {
        let now = Utc::now();
        let unix_timestamp = now.timestamp();

        trace!(
            "Updating indexer progress: task_id={}, value={}",
            task_id,
            value
        );

        match self
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK", AttributeValue::S(String::from("INDEXER")))
            .key("SK", AttributeValue::S(format!("TASK#{}", task_id)))
            .update_expression(
                "SET indexation_progress = :indexation_progress, last_update = :last_update",
            )
            .expression_attribute_values(
                ":indexation_progress",
                AttributeValue::N(value.to_string()),
            )
            .expression_attribute_values(
                ":last_update",
                AttributeValue::N(unix_timestamp.to_string()),
            )
            .send()
            .await
        {
            Ok(output) => {
                debug!("Upsert operation successful: {:?}", output);
                Ok(())
            }
            Err(error) => {
                error!(
                    "Upsert operation failed for task_id {}: {:?}",
                    task_id, error
                );
                Err(anyhow!(error)
                    .context(format!("Failed to update progress for task_id {}", task_id)))
            }
        }
    }
}

#[async_trait]
impl Storage for DynamoStorage {
    async fn register_mint(
        &self,
        _token: &TokenFromEvent,
        _block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn register_token(
        &self,
        _token: &TokenFromEvent,
        _block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn register_event(
        &self,
        _event: &TokenEvent,
        _block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn get_contract_type(
        &self,
        _contract_address: &FieldElement,
    ) -> Result<ContractType, StorageError> {
        Ok(ContractType::ERC721)
    }

    async fn register_contract_info(
        &self,
        _contract_address: &FieldElement,
        _contract_type: &ContractType,
        _block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn set_block_info(
        &self,
        _block_number: u64,
        _info: BlockInfo,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn get_block_info(&self, _block_number: u64) -> Result<BlockInfo, StorageError> {
        Ok(BlockInfo {
            indexer_version: 0,
            indexer_identifier: String::from("test"),
            status: BlockIndexingStatus::Processing,
        })
    }

    async fn clean_block(&self, _block_number: u64) -> Result<(), StorageError> {
        Ok(())
    }

    async fn update_last_pending_block(
        &self,
        _block_number: u64,
        _block_timestamp: u64,
    ) -> Result<(), StorageError> {
        // TODO: when this is called, we've successfully process the `pending`
        // block that became the `latest`.
        // So we should update the storage with the new block number
        // based on the timestamp to identify the block.
        Ok(())
    }
}

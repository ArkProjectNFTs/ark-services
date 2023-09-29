use anyhow::{anyhow, Result};
use arkproject::pontos::storage::{
    types::{
        BlockIndexingStatus, BlockInfo, ContractType, IndexerStatus, StorageError, TokenEvent,
        TokenFromEvent,
    },
    Storage,
};
use async_trait::async_trait;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::Utc;
use log::{debug, error, trace};
use starknet::core::types::FieldElement;

pub struct DynamoStorage {
    client: Client,
}

impl DynamoStorage {
    pub fn new(client: Client) -> Self {
        Self { client }
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

const TABLE_NAME: &str = "ark-test";

#[async_trait]
impl AWSDynamoStorage for DynamoStorage {
    async fn update_indexer_task_status(
        &self,
        task_id: String,
        indexer_version: u64,
        status: IndexerStatus,
    ) -> Result<()> {
        let table_name = String::from("ark-test");

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
            .table_name(TABLE_NAME)
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
        let table_name = String::from("ark-test");

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
            .table_name(TABLE_NAME)
            .key("PK", AttributeValue::S(String::from("INDEXER")))
            .update_expression("SET indexation_progress = :indexation_progress")
            .expression_attribute_values(
                ":indexation_progress",
                AttributeValue::N(value.to_string()),
            )
            .send()
            .await
        {
            Ok(output) => {
                debug!("Upsert operation successful: {:?}", output);
                Ok(())
            }
            Err(error) => {
                debug!(
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
        token: &TokenFromEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn register_token(
        &self,
        token: &TokenFromEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn register_event(
        &self,
        event: &TokenEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn get_contract_type(
        &self,
        contract_address: &FieldElement,
    ) -> Result<ContractType, StorageError> {
        Ok(ContractType::ERC721)
    }

    async fn register_contract_info(
        &self,
        contract_address: &FieldElement,
        contract_type: &ContractType,
        block_number: u64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    async fn set_block_info(&self, block_number: u64, info: BlockInfo) -> Result<(), StorageError> {
        Ok(())
    }

    async fn get_block_info(&self, block_number: u64) -> Result<BlockInfo, StorageError> {
        Ok(BlockInfo {
            indexer_version: 0,
            indexer_identifier: String::from("test"),
            status: BlockIndexingStatus::Processing,
        })
    }

    async fn clean_block(&self, block_number: u64) -> Result<(), StorageError> {
        Ok(())
    }
}

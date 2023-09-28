use arkproject::pontos::storage::{
    types::{
        BlockIndexing, BlockIndexingStatus, BlockInfo, ContractType, IndexerStatus, StorageError,
        TokenEvent, TokenFromEvent,
    },
    Storage,
};

use async_trait::async_trait;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::Utc;
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

impl DynamoStorage {
    async fn set_indexer_progress(&self, progress: BlockIndexing) -> Result<(), StorageError> {
        let task_id = progress.identifier;
        let status = match progress.status {
            IndexerStatus::Running => String::from("running"),
            IndexerStatus::Stopped => String::from("stopped"),
        };
        let now = Utc::now();
        let unix_timestamp = now.timestamp();

        self.client
            .put_item()
            .table_name(String::from(""))
            .item("PK", AttributeValue::S(String::from("INDEXER")))
            .item("SK", AttributeValue::S(format!("TASK#{}", task_id)))
            .item("status", AttributeValue::S(status))
            .item("last_update", AttributeValue::N(unix_timestamp.to_string()))
            .item(
                "version",
                AttributeValue::N(progress.indexer_version.to_string()),
            )
            .item("task_id", AttributeValue::S(task_id.to_string()))
            .item("from", AttributeValue::N(progress.range.start.to_string()))
            .item("to", AttributeValue::N(progress.range.end.to_string()))
            .item(
                "indexation_progress",
                AttributeValue::N(progress.percentage.to_string()),
            )
            .send()
            .await
            .map_err(|_| StorageError::DatabaseError)?;

        Ok(())
    }
}

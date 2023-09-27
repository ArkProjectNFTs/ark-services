use arkproject::pontos::storage::{
    types::{
        BlockIndexing, BlockIndexingStatus, BlockInfo, ContractType, StorageError, TokenEvent,
        TokenFromEvent,
    },
    StorageManager,
};
use async_trait::async_trait;
use starknet::core::types::FieldElement;

#[derive(Default)]
pub struct DynamoStorage {

    // TODO: add DynamoDB client.

}

#[async_trait]
impl StorageManager for DynamoStorage {
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
        Ok(())
    }
}
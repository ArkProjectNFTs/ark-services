
use arkproject::{
    metadata::{
        storage::Storage,
        types::{StorageError, TokenMetadata},
    },
    starknet::CairoU256,
};
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use starknet::core::types::FieldElement;
use anyhow::Result;
use aws_config::load_from_env;

pub struct MetadataStorage {
    client: Client,
    table_name: String,
}

impl MetadataStorage {
    pub async fn new(table_name: String) -> Self {
        let config = load_from_env().await;
        let client = Client::new(&config);
        Self { client, table_name }
    }
}

#[async_trait]
impl Storage for MetadataStorage {

    fn register_token_metadata(
        &self,
        contract_address: &FieldElement,
        token_id: CairoU256,
        token_metadata: TokenMetadata,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    fn has_token_metadata(
        &self,
        contract_address: FieldElement,
        token_id: CairoU256,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn find_token_ids_without_metadata_in_collection(
        &self,
        contract_address: FieldElement,
    ) -> Result<Vec<CairoU256>, StorageError> {
        Ok(vec![])
    }

    fn find_token_ids_without_metadata(
        &self,
    ) -> Result<Vec<(FieldElement, CairoU256)>, StorageError> {
        let result: Vec<(FieldElement, CairoU256)> = vec![(FieldElement::ONE, CairoU256 { high: 0, low: 0 })];
        Ok(result)
    }
}

use crate::{providers::ArkTokenProvider, ArkDynamoDbProvider, DynamoDbCtx};
use anyhow::Result;
use arkproject::{
    metadata::{
        storage::Storage,
        types::{StorageError, TokenMetadata},
    },
    starknet::CairoU256,
};
use async_trait::async_trait;
use aws_config::load_from_env;
use aws_sdk_dynamodb::Client;
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use tracing::error;

pub struct MetadataStorage {
    ctx: DynamoDbCtx,
    provider: ArkDynamoDbProvider,
}

impl MetadataStorage {
    pub async fn new(table_name: String) -> Self {
        let config = load_from_env().await;
        let client = Client::new(&config);
        let ctx = DynamoDbCtx {
            client,
            exclusive_start_key: None,
            multiple_exclusive_start_keys: HashMap::new(),
        };

        // Internally, we want more items to be loaded until reaching 1MB.
        let limit = Some(1000);
        let provider = ArkDynamoDbProvider::new(&table_name, limit);
        Self { ctx, provider }
    }
}

#[async_trait]
impl Storage for MetadataStorage {
    async fn update_token_metadata_status(
        &self,
        contract_address: FieldElement,
        token_id: CairoU256,
        metadata_status: &str,
    ) -> Result<(), StorageError> {
        let token_id_hex = token_id.to_hex();
        let contract_address_hex = format!("0x{:064x}", contract_address);

        self.provider
            .token
            .update_token_metadata_status(
                &self.ctx,
                &contract_address_hex,
                &token_id_hex,
                metadata_status,
            )
            .await
            .map_err(|e| {
                error!("Failed to update token metadata status. Error: {}", e);
                StorageError::DatabaseError(format!("{:?}", e.to_string()))
            })?;

        Ok(())
    }

    async fn register_token_metadata(
        &self,
        contract_address: &FieldElement,
        token_id: CairoU256,
        token_metadata: TokenMetadata,
    ) -> Result<(), StorageError> {
        let token_id_hex = token_id.to_hex();
        let contract_address_hex = format!("0x{:064x}", contract_address);

        let result = self
            .provider
            .token
            .update_metadata(
                &self.ctx,
                contract_address_hex.as_str(),
                token_id_hex.clone().as_str(),
                &token_metadata,
            )
            .await;

        match result {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                error!("{}", e.to_string());
                return Err(StorageError::DatabaseError(format!("{:?}", e.to_string())));
            }
        }
    }

    async fn has_token_metadata(
        &self,
        _contract_address: FieldElement,
        _token_id: CairoU256,
    ) -> Result<bool, StorageError> {
        Err(StorageError::DatabaseError("Not implemented".to_string()))
    }

    async fn find_token_ids_without_metadata(
        &self,
        contract_address_filter: Option<FieldElement>,
    ) -> Result<Vec<(FieldElement, CairoU256)>, StorageError> {
        match self
            .provider
            .token
            .get_token_without_metadata(&self.ctx.client, contract_address_filter)
            .await
        {
            Ok(tokens) => {
                return Ok(tokens);
            }
            Err(e) => Err(StorageError::DatabaseError(format!("{:?}", e.to_string()))),
        }
    }
}

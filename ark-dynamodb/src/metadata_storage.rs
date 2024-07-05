use crate::{providers::ArkTokenProvider, ArkDynamoDbProvider, DynamoDbCtx};
use anyhow::Result;
use arkproject::metadata::{
    storage::Storage,
    types::{StorageError, TokenMetadata, TokenWithoutMetadata},
};
use async_trait::async_trait;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::Client;
use num_bigint::BigUint;
use std::{collections::HashMap, str::FromStr};
use tracing::error;

pub struct MetadataStorage {
    ctx: DynamoDbCtx,
    provider: ArkDynamoDbProvider,
}

impl MetadataStorage {
    pub async fn new(table_name: String) -> Self {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
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
    async fn update_all_token_metadata_status(
        &self,
        _contract_address: &str,
        _chain_id: &str,
        _metadata_status: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::DatabaseError("Not implemented".to_string()))
    }

    async fn set_contract_refreshing_status(
        &self,
        _contract_address: &str,
        _chain_id: &str,
        _is_refreshing: bool,
    ) -> Result<(), StorageError> {
        Err(StorageError::DatabaseError("Not implemented".to_string()))
    }

    async fn update_token_metadata_status(
        &self,
        contract_address: &str,
        token_id: &str,
        _chain_id: &str,
        metadata_status: &str,
    ) -> Result<(), StorageError> {
        let token_id_bn = match BigUint::from_str(token_id) {
            Ok(id) => id,
            Err(_) => return Err(StorageError::DatabaseError("Invalid token id".to_string())),
        };

        let token_id_hex = token_id_bn.to_str_radix(16);

        self.provider
            .token
            .update_token_metadata_status(
                &self.ctx,
                contract_address,
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
        contract_address: &str,
        token_id: &str,
        _chain_id: &str,
        token_metadata: TokenMetadata,
    ) -> Result<(), StorageError> {
        let token_id_bn = match BigUint::from_str(token_id) {
            Ok(id) => id,
            Err(_) => return Err(StorageError::DatabaseError("Invalid token id".to_string())),
        };

        let token_id_hex = token_id_bn.to_str_radix(16);

        let result = self
            .provider
            .token
            .update_metadata(&self.ctx, contract_address, &token_id_hex, &token_metadata)
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

    async fn find_tokens_without_metadata(
        &self,
        filter: Option<(String, String)>,
        _refresh_collection: bool,
    ) -> Result<Vec<TokenWithoutMetadata>, StorageError> {
        match self
            .provider
            .token
            .get_token_without_metadata(&self.ctx.client, filter)
            .await
        {
            Ok(tokens) => Ok(tokens),
            Err(e) => Err(StorageError::DatabaseError(format!("{:?}", e.to_string()))),
        }
    }
}

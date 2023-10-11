use anyhow::Result;
use arkproject::pontos::storage::{
    types::{
        BlockInfo, ContractInfo, ContractType, IndexerStatus, StorageError, TokenEvent, TokenInfo,
    },
    Storage,
};
use async_trait::async_trait;
use aws_config::load_from_env;
use aws_sdk_dynamodb::{
    types::{AttributeValue, DeleteRequest, ReturnValue, WriteRequest},
    Client,
};
use chrono::Utc;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::{debug, error, info};

use crate::providers::{ArkBlockProvider, ArkContractProvider, ArkEventProvider, ArkTokenProvider};
use crate::ArkDynamoDbProvider;

pub struct DynamoStorage {
    client: Client,
    table_name: String,
    provider: ArkDynamoDbProvider,
}

impl DynamoStorage {
    pub async fn new(table_name: String) -> Self {
        let config = load_from_env().await;
        let client = Client::new(&config);
        let provider = ArkDynamoDbProvider::new(&table_name);
        Self {
            client,
            table_name,
            provider,
        }
    }
}

#[async_trait]
pub trait AWSDynamoStorage: Send + Sync {
    async fn update_indexer_task_status(
        &self,
        task_id: String,
        indexer_version: String,
        status: IndexerStatus,
    ) -> Result<(), StorageError>;
    async fn update_indexer_progress(
        &self,
        task_id: String,
        value: f64,
    ) -> Result<(), StorageError>;
}

#[async_trait]
impl AWSDynamoStorage for DynamoStorage {
    async fn update_indexer_task_status(
        &self,
        task_id: String,
        indexer_version: String,
        status: IndexerStatus,
    ) -> Result<(), StorageError> {
        let now = Utc::now();
        let unix_timestamp = now.timestamp();

        let status_string = match status {
            IndexerStatus::Requested => "requested",
            IndexerStatus::Running => "running",
            IndexerStatus::Stopped => "stopped",
        }
        .to_string();

        let mut data = HashMap::new();
        data.insert(
            "status".to_string(),
            AttributeValue::S(status_string.clone()),
        );
        data.insert(
            "last_update".to_string(),
            AttributeValue::N(unix_timestamp.to_string()),
        );
        data.insert(
            "version".to_string(),
            AttributeValue::S(indexer_version.clone()),
        );
        data.insert("task_id".to_string(), AttributeValue::S(task_id.clone()));

        let response = self
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(format!("INDEXER#{}", task_id)))
            .item("SK", AttributeValue::S("TASK".to_string()))
            .item("Type", AttributeValue::S("IndexerTask".to_string()))
            .item("Data", AttributeValue::M(data))
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
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn update_indexer_progress(
        &self,
        task_id: String,
        value: f64,
    ) -> Result<(), StorageError> {
        let now = Utc::now();
        let unix_timestamp = now.timestamp();

        info!(
            "Updating indexer progress: task_id={}, value={}",
            task_id, value
        );

        let mut values = HashMap::new();
        values.insert(
            ":IndexationProgress".to_string(),
            AttributeValue::N(value.to_string()),
        );
        values.insert(
            ":LastUpdate".to_string(),
            AttributeValue::S(unix_timestamp.to_string()),
        );

        let mut names = HashMap::new();
        names.insert("#Data".to_string(), "Data".to_string());
        names.insert(
            "#IndexationProgress".to_string(),
            "IndexationProgress".to_string(),
        );
        names.insert("#LastUpdate".to_string(), "LastUpdate".to_string());

        let response = self
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK", AttributeValue::S("INDEXER".to_string()))
            .key("SK", AttributeValue::S(format!("TASK#{}", task_id)))
            .update_expression(
                "SET #Data.#IndexationProgress = :IndexationProgress, #Data.#LastUpdate = :LastUpdate",
            )
            .set_expression_attribute_names(Some(names))
            .set_expression_attribute_values(Some(values))
            .return_values(ReturnValue::AllNew)
            .send()
            .await;

        match response {
            Ok(_) => {
                info!(
                    "Successfully updated indexer progress for task_id={}: indexation_progress={}",
                    task_id, value
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to update indexer progress for task_id {}: {:?}",
                    task_id, e
                );
                Err(StorageError::DatabaseError)
            }
        }
    }
}

#[async_trait]
impl Storage for DynamoStorage {
    async fn register_mint(
        &self,
        token: &TokenInfo,
        block_number: u64,
    ) -> Result<(), StorageError> {
        info!("Registering mint {:?}", token);

        let info_existing = match self
            .provider
            .token
            .get_token(&self.client, &token.contract_address, &token.token_id_hex)
            .await
        {
            Ok(i) => i,
            Err(e) => {
                error!("Registering mint error {}", e.to_string());
                return Err(StorageError::DatabaseError);
            }
        };

        if let Some(mut info) = info_existing {
            // Token already exists, add mint info.
            info.mint_address = token.mint_address.clone();
            info.mint_transaction_hash = token.mint_transaction_hash.clone();
            info.mint_timestamp = token.mint_timestamp;
            info.mint_block_number = Some(block_number);

            match self
                .provider
                .token
                .update_mint_data(&self.client, &info)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("{}", e.to_string());
                    return Err(StorageError::DatabaseError);
                }
            }
        } else {
            // Token does not exist, create it with mint info.
            let info = TokenInfo {
                mint_address: token.mint_address.clone(),
                mint_transaction_hash: token.mint_transaction_hash.clone(),
                mint_timestamp: token.mint_timestamp,
                mint_block_number: Some(block_number),
                owner: token.owner.clone(),
                contract_address: token.contract_address.clone(),
                token_id: token.token_id.clone(),
                token_id_hex: token.token_id_hex.clone(),
            };

            match self
                .provider
                .token
                .register_token(&self.client, &info, block_number)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("{}", e.to_string());
                    return Err(StorageError::DatabaseError);
                }
            }
        }
    }

    async fn register_token(
        &self,
        token: &TokenInfo,
        block_number: u64,
    ) -> Result<(), StorageError> {
        debug!("Registering token {:?}", token);

        let does_exist = self
            .provider
            .token
            .get_token(&self.client, &token.contract_address, &token.token_id_hex)
            .await
            .map_err(|_| StorageError::DatabaseError)?
            .is_some();

        if does_exist {
            match self
                .provider
                .token
                .update_owner(
                    &self.client,
                    &token.contract_address,
                    &token.token_id_hex,
                    &token.owner,
                )
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("{}", e.to_string());
                    return Err(StorageError::DatabaseError);
                }
            }
        } else {
            // Create the full token entry.
            let info = TokenInfo {
                owner: token.owner.clone(),
                contract_address: token.contract_address.clone(),
                token_id: token.token_id.clone(),
                token_id_hex: token.token_id_hex.clone(),
                ..Default::default()
            };

            match self
                .provider
                .token
                .register_token(&self.client, &info, block_number)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("{}", e.to_string());
                    return Err(StorageError::DatabaseError);
                }
            }
        }
    }

    async fn register_event(
        &self,
        event: &TokenEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        info!("Registering event {:?}", event);

        let info = match self
            .provider
            .event
            .get_event(&self.client, &event.contract_address, &event.event_id)
            .await
        {
            Ok(i) => i,
            Err(e) => {
                error!("{}", e.to_string());
                return Err(StorageError::DatabaseError);
            }
        };

        if info.is_some() {
            return Err(StorageError::AlreadyExists);
        }

        match self
            .provider
            .event
            .register_event(&self.client, event, block_number)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e.to_string());
                return Err(StorageError::DatabaseError);
            }
        }
    }

    async fn get_contract_type(
        &self,
        contract_address: &str,
    ) -> Result<ContractType, StorageError> {
        info!("Getting contract info for contract {}", contract_address);

        match self
            .provider
            .contract
            .get_contract(&self.client, contract_address)
            .await
        {
            Ok(maybe_contract) => {
                if let Some(contract) = maybe_contract {
                    // unwrap should be safe here as the type is controlled by
                    // the `ContractInfo` directly.
                    Ok(ContractType::from_str(&contract.contract_type).unwrap())
                } else {
                    return Err(StorageError::NotFound);
                }
            }
            Err(e) => {
                error!("{}", e.to_string());
                return Err(StorageError::DatabaseError);
            }
        }
    }

    async fn register_contract_info(&self, info: &ContractInfo) -> Result<(), StorageError> {
        info!(
            "Registering contract info {:?} for contract {}",
            info.contract_type, info.contract_address
        );

        match self
            .provider
            .contract
            .register_contract(&self.client, info)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e.to_string());
                return Err(StorageError::DatabaseError);
            }
        }
    }

    async fn set_block_info(&self, block_number: u64, info: BlockInfo) -> Result<(), StorageError> {
        info!("Setting block info {:?} for block #{}", info, block_number);

        match self
            .provider
            .block
            .set_info(&self.client, block_number, &info)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e.to_string());
                return Err(StorageError::DatabaseError);
            }
        }
    }

    async fn get_block_info(&self, block_number: u64) -> Result<BlockInfo, StorageError> {
        info!("Getting block info for block #{}", block_number);

        let info = match self
            .provider
            .block
            .get_info(&self.client, block_number)
            .await
        {
            Ok(i) => i,
            Err(e) => {
                error!("{}", e.to_string());
                return Err(StorageError::DatabaseError);
            }
        };

        if let Some(info) = info {
            Ok(info)
        } else {
            Err(StorageError::NotFound)
        }
    }

    async fn clean_block(&self, block_number: u64) -> Result<(), StorageError> {
        info!("Cleaning block #{}", block_number);
        let table_name = self.table_name.clone();
        let gsi_pk = format!("BLOCK#{}", block_number);

        // Query for all items associated with the block number
        let query_output = self
            .client
            .query()
            .table_name(&table_name)
            .index_name("GSI4PK-GSI4SK-index") // Assuming your GSI for block association is named GSI4
            .key_condition_expression("GSI4PK = :gsi_pk")
            .expression_attribute_values(":gsi_pk", AttributeValue::S(gsi_pk))
            .projection_expression("PK, SK") // Only retrieve necessary attributes
            .send()
            .await
            .map_err(|e| {
                eprintln!("Query error: {:?}", e);
                StorageError::DatabaseError
            })?;

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
            let batch_write_output = self
                .client
                .batch_write_item()
                .request_items(&table_name, chunk.to_vec())
                .send()
                .await
                .map_err(|e| {
                    error!("Batch write error: {:?}", e);
                    StorageError::DatabaseError
                })?;

            // Handle unprocessed items if any
            if let Some(unprocessed_items) = batch_write_output.unprocessed_items {
                if let Some(retry_items) = unprocessed_items.get(&table_name) {
                    // Implement retry logic as per your use case
                    // Here, we'll simply wait for a second and try again
                    sleep(Duration::from_secs(1)).await;
                    self.client
                        .batch_write_item()
                        .request_items(&table_name, retry_items.clone())
                        .send()
                        .await
                        .map_err(|e| {
                            error!("Retry batch write error: {:?}", e);
                            StorageError::DatabaseError
                        })?;
                }
            }
        }

        // Delete the block entry
        let pk_block = format!("BLOCK#{}", block_number);
        let sk_block = "BLOCK".to_string();
        self.client
            .delete_item()
            .table_name(&table_name)
            .key("PK", AttributeValue::S(pk_block))
            .key("SK", AttributeValue::S(sk_block))
            .send()
            .await
            .map_err(|e| {
                error!("Delete block entry error: {:?}", e);
                StorageError::DatabaseError
            })?;

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

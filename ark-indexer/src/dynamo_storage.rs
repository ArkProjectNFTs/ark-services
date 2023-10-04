use anyhow::Result;
use arkproject::pontos::storage::{
    types::{
        BlockIndexingStatus, BlockInfo, ContractType, IndexerStatus, StorageError, TokenEvent,
        TokenFromEvent,
    },
    Storage,
};

use arkproject::starknet::format::to_hex_str;
use async_trait::async_trait;
use aws_config::load_from_env;
use aws_sdk_dynamodb::{
    types::AttributeValue, types::DeleteRequest, types::ReturnValue, types::WriteRequest, Client,
};
use chrono::Utc;
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use std::fmt;
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::{debug, error, info};

#[derive(Debug, PartialEq, Eq)]
enum EntityType {
    Token,
    Block,
    Collection,
    Event,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Token => write!(f, "Token"),
            EntityType::Block => write!(f, "Block"),
            EntityType::Collection => write!(f, "Collection"),
            EntityType::Event => write!(f, "Event"),
        }
    }
}

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

        let mut data = HashMap::new();
        data.insert("status".to_string(), AttributeValue::S(value.to_string()));
        data.insert(
            "last_update".to_string(),
            AttributeValue::N(unix_timestamp.to_string()),
        );

        let response = self
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key("PK", AttributeValue::S(format!("INDEXER#{}", task_id)))
            .key("SK", AttributeValue::S("TASK".to_string()))
            .update_expression("SET #Data = :data")
            .expression_attribute_names("#Data", "Data")
            .expression_attribute_values(":data".to_string(), AttributeValue::M(data))
            .return_values(ReturnValue::AllNew)
            .send()
            .await;

        match response {
            Ok(_) => {
                debug!(
                    "Successfully updated indexer progress for task_id {}: value {}",
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
        token: &TokenFromEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        info!("Registering mint {:?}", token);

        // Construct the primary key for the token
        let pk = format!(
            "TOKEN#{}#{}",
            token.address, token.formated_token_id.token_id
        );
        let sk = "TOKEN".to_string();

        // Check if the token already exists in DynamoDB
        let get_item_output = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk.clone()))
            .key("SK".to_string(), AttributeValue::S(sk.clone()))
            .send()
            .await;

        match get_item_output {
            Ok(output) if output.item.is_some() => {
                // Update existing item in case we indexed a transfer before the mint
                let mut data_map = HashMap::new();
                data_map.insert("Owner".to_string(), AttributeValue::S(token.owner.clone()));
                data_map.insert(
                    "MintAddress".to_string(),
                    AttributeValue::S(to_hex_str(&token.mint_address.unwrap())),
                );
                data_map.insert(
                    "MintTimestamp".to_string(),
                    AttributeValue::N(token.mint_timestamp.unwrap().to_string()),
                );
                data_map.insert(
                    "BlockNumber".to_string(),
                    AttributeValue::N(block_number.to_string()),
                );

                let update_item_output = self
                    .client
                    .update_item()
                    .table_name(self.table_name.clone())
                    .key("PK".to_string(), AttributeValue::S(pk.clone()))
                    .key("SK".to_string(), AttributeValue::S(sk.clone()))
                    .update_expression("SET Data = :data")
                    .expression_attribute_values(":data".to_string(), AttributeValue::M(data_map))
                    .return_values(ReturnValue::AllNew)
                    .send()
                    .await;

                match update_item_output {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("DynamoDB error: {:?}", e);
                        Err(StorageError::DatabaseError)
                    }
                }
            }
            Ok(_) => {
                // Create new item for the minted token
                let mut data_map = HashMap::new();
                data_map.insert(
                    "BlockNumber".to_string(),
                    AttributeValue::N(block_number.to_string()),
                );
                data_map.insert("Owner".to_string(), AttributeValue::S(token.owner.clone()));
                data_map.insert(
                    "contract_address".to_string(),
                    AttributeValue::S(token.address.clone()),
                );
                data_map.insert(
                    "TokenId".to_string(),
                    AttributeValue::S(token.formated_token_id.token_id.to_string()),
                );
                data_map.insert(
                    "MintAddress".to_string(),
                    AttributeValue::S(to_hex_str(&token.mint_address.unwrap())),
                );
                data_map.insert(
                    "MintTimestamp".to_string(),
                    AttributeValue::N(token.mint_timestamp.unwrap().to_string()),
                );

                let put_item_output = self
                    .client
                    .put_item()
                    .table_name(self.table_name.clone())
                    .item("PK".to_string(), AttributeValue::S(pk.clone()))
                    .item("SK".to_string(), AttributeValue::S(sk.clone()))
                    .item("Type".to_string(), AttributeValue::S("Token".to_string()))
                    .item(
                        "GSI1PK".to_string(),
                        AttributeValue::S(format!("COLLECTION#{}", token.address)),
                    )
                    .item(
                        "GSI1SK".to_string(),
                        AttributeValue::S(format!("TOKEN#{}", token.formated_token_id.token_id)),
                    )
                    .item(
                        "GSI2PK".to_string(),
                        AttributeValue::S(format!("OWNER#{}", token.owner)),
                    )
                    .item(
                        "GSI2SK".to_string(),
                        AttributeValue::S(format!(
                            "TOKEN#{}#{}",
                            token.address, token.formated_token_id.token_id
                        )),
                    )
                    .item(
                        "GSI3PK".to_string(),
                        AttributeValue::S("LISTED#false".to_string()),
                    ) // Assuming the token is listed by default
                    .item(
                        "GSI3SK".to_string(),
                        AttributeValue::S(format!(
                            "TOKEN#{}#{}",
                            token.address, token.formated_token_id.padded_token_id
                        )),
                    )
                    .item(
                        "GSI4PK".to_string(),
                        AttributeValue::S(format!("BLOCK#{}", block_number)),
                    )
                    .item(
                        "GSI4SK".to_string(),
                        AttributeValue::S(format!(
                            "TOKEN#{}#{}",
                            token.address, token.formated_token_id.padded_token_id
                        )),
                    )
                    .item("Data".to_string(), AttributeValue::M(data_map))
                    .item("Type", AttributeValue::S(EntityType::Token.to_string()))
                    .return_values(ReturnValue::AllOld)
                    .send()
                    .await;

                match put_item_output {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("DynamoDB error: {:?}", e);
                        Err(StorageError::DatabaseError)
                    }
                }
            }
            Err(e) => {
                error!("DynamoDB error: {:?}", e);
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn register_token(
        &self,
        token: &TokenFromEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        debug!("Registering token {:?}", token);

        // Construct the primary key and secondary key for the token
        let pk = format!(
            "TOKEN#{}#{}",
            token.address, token.formated_token_id.token_id
        );
        let sk = "TOKEN".to_string();

        // Check if the token already exists in DynamoDB
        let get_item_output = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk.clone()))
            .key("SK".to_string(), AttributeValue::S(sk.clone()))
            .send()
            .await;

        match get_item_output {
            Ok(output) if output.item.is_some() => {
                // Update existing item
                let mut data_map = HashMap::new();
                data_map.insert("Owner".to_string(), AttributeValue::S(token.owner.clone()));

                let update_item_output = self
                    .client
                    .update_item()
                    .table_name(self.table_name.clone())
                    .key("PK".to_string(), AttributeValue::S(pk.clone()))
                    .key("SK".to_string(), AttributeValue::S(sk.clone()))
                    .update_expression("SET Data = :data")
                    .expression_attribute_values(":data".to_string(), AttributeValue::M(data_map))
                    .return_values(ReturnValue::AllNew)
                    .send()
                    .await;

                match update_item_output {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("DynamoDB error: {:?}", e);
                        Err(StorageError::DatabaseError)
                    }
                }
            }
            Ok(_) => {
                // Create new item
                let mut data_map = HashMap::new();
                data_map.insert(
                    "BlockNumber".to_string(),
                    AttributeValue::N(block_number.to_string()),
                );
                data_map.insert("Owner".to_string(), AttributeValue::S(token.owner.clone()));
                // TODO move as a GSIPK to do quick lookups
                data_map.insert(
                    "contract_address".to_string(),
                    AttributeValue::S(token.address.clone()),
                );
                data_map.insert(
                    "TokenId".to_string(),
                    AttributeValue::S(token.formated_token_id.token_id.to_string()),
                );

                let put_item_output = self
                    .client
                    .put_item()
                    .table_name(self.table_name.clone())
                    .item("PK".to_string(), AttributeValue::S(pk.clone()))
                    .item("SK".to_string(), AttributeValue::S(sk.clone()))
                    .item("Type".to_string(), AttributeValue::S("Token".to_string()))
                    .item(
                        "GSI1PK".to_string(),
                        AttributeValue::S(format!("COLLECTION#{}", token.address)),
                    )
                    .item(
                        "GSI1SK".to_string(),
                        AttributeValue::S(format!("TOKEN#{}", token.formated_token_id.token_id)),
                    )
                    .item(
                        "GSI2PK".to_string(),
                        AttributeValue::S(format!("OWNER#{}", token.owner)),
                    )
                    .item(
                        "GSI2SK".to_string(),
                        AttributeValue::S(format!(
                            "TOKEN#{}#{}",
                            token.address, token.formated_token_id.token_id
                        )),
                    )
                    .item(
                        "GSI3PK".to_string(),
                        AttributeValue::S("LISTED#false".to_string()),
                    ) // Assuming the token is listed by default
                    .item(
                        "GSI3SK".to_string(),
                        AttributeValue::S(format!(
                            "TOKEN#{}#{}",
                            token.address, token.formated_token_id.padded_token_id
                        )),
                    )
                    .item(
                        "GSI4PK".to_string(),
                        AttributeValue::S(format!("BLOCK#{}", block_number)),
                    )
                    .item(
                        "GSI4SK".to_string(),
                        AttributeValue::S(format!(
                            "TOKEN#{}#{}",
                            token.address, token.formated_token_id.padded_token_id
                        )),
                    )
                    .item("Data".to_string(), AttributeValue::M(data_map))
                    .item("Type", AttributeValue::S(EntityType::Token.to_string()))
                    .return_values(ReturnValue::AllOld)
                    .send()
                    .await;

                match put_item_output {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("DynamoDB error: {:?}", e);
                        Err(StorageError::DatabaseError)
                    }
                }
            }
            Err(e) => {
                error!("DynamoDB error: {:?}", e);
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn register_event(
        &self,
        event: &TokenEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        info!("Registering event {:?}", event);

        // Construct the primary key and secondary key for the event
        let pk = format!(
            "EVENT#{}#{}",
            event.contract_address,
            to_hex_str(&event.event_id)
        );
        let sk = "EVENT".to_string();

        // Check if the event already exists in DynamoDB
        let get_item_output = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk.clone()))
            .key("SK".to_string(), AttributeValue::S(sk.clone()))
            .send()
            .await;

        match get_item_output {
            Ok(output) if output.item.is_some() => Err(StorageError::AlreadyExists),
            Ok(_) => {
                // Create new item
                let mut data_map = HashMap::new();
                data_map.insert(
                    "Timestamp".to_string(),
                    AttributeValue::N(event.timestamp.to_string()),
                );
                data_map.insert(
                    "FromAddress".to_string(),
                    AttributeValue::S(to_hex_str(&event.from_address_field_element)),
                );
                data_map.insert(
                    "ToAddress".to_string(),
                    AttributeValue::S(to_hex_str(&event.to_address_field_element)),
                );
                data_map.insert(
                    "ContractAddress".to_string(),
                    AttributeValue::S(event.contract_address.clone()),
                );
                data_map.insert(
                    "TransactionHash".to_string(),
                    AttributeValue::S(event.transaction_hash.clone()),
                );
                data_map.insert(
                    "TokenId".to_string(),
                    AttributeValue::S(event.formated_token_id.token_id.clone()),
                );
                data_map.insert(
                    "BlockNumber".to_string(),
                    AttributeValue::N(event.block_number.to_string()),
                );
                data_map.insert(
                    "ContractType".to_string(),
                    AttributeValue::S(event.contract_type.clone().to_string()),
                );
                data_map.insert(
                    "EventType".to_string(),
                    AttributeValue::S(event.event_type.clone().to_string()),
                );
                data_map.insert(
                    "EventId".to_string(),
                    AttributeValue::S(to_hex_str(&event.event_id)),
                );
                data_map.insert(
                    "BlockNumber".to_string(),
                    AttributeValue::N(block_number.to_string()),
                );

                let put_item_output = self
                    .client
                    .put_item()
                    .table_name(self.table_name.clone())
                    .item("PK".to_string(), AttributeValue::S(pk.clone()))
                    .item("SK".to_string(), AttributeValue::S(sk.clone()))
                    .item("Type".to_string(), AttributeValue::S("Event".to_string()))
                    .item(
                        "GSI1PK".to_string(),
                        AttributeValue::S(format!("COLLECTION#{}", event.contract_address)),
                    )
                    .item(
                        "GSI1SK".to_string(),
                        AttributeValue::S(format!("EVENT#{}", event.event_id)),
                    )
                    .item(
                        "GSI2PK".to_string(),
                        AttributeValue::S(format!("TOKEN#{}", event.formated_token_id.token_id)),
                    )
                    .item(
                        "GSI2SK".to_string(),
                        AttributeValue::S(format!("EVENT#{}", event.event_id)),
                    )
                    .item(
                        "GSI4PK".to_string(),
                        AttributeValue::S(format!("BLOCK#{}", block_number)),
                    )
                    .item(
                        "GSI4SK".to_string(),
                        AttributeValue::S(format!(
                            "EVENT#{}#{}",
                            event.contract_address, event.event_id
                        )),
                    )
                    .item("Data".to_string(), AttributeValue::M(data_map))
                    .item("Type", AttributeValue::S(EntityType::Event.to_string()))
                    .return_values(ReturnValue::AllOld)
                    .send()
                    .await;

                match put_item_output {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("DynamoDB error: {:?}", e);
                        Err(StorageError::DatabaseError)
                    }
                }
            }
            Err(e) => {
                error!("DynamoDB error: {:?}", e);
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn get_contract_type(
        &self,
        contract_address: &FieldElement,
    ) -> Result<ContractType, StorageError> {
        info!("Getting contract info for contract {}", contract_address);

        // Construct the primary key and secondary key for the contract
        let pk = format!("COLLECTION#{}", to_hex_str(contract_address));
        let sk = "COLLECTION".to_string();

        // Fetch the contract from DynamoDB
        let get_item_output = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), AttributeValue::S(pk))
            .key("SK".to_string(), AttributeValue::S(sk))
            .send()
            .await;

        match get_item_output {
            Ok(output) => {
                if let Some(item) = output.item {
                    if let Some(AttributeValue::M(data)) = item.get("Data") {
                        if let Some(AttributeValue::S(contract_type_str)) =
                            data.get("contract_type")
                        {
                            let contract_type: ContractType = contract_type_str.parse().unwrap();
                            return Ok(contract_type);
                        }
                    }
                }
                Err(StorageError::NotFound)
            }
            Err(e) => {
                error!("DynamoDB error: {:?}", e);
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn register_contract_info(
        &self,
        contract_address: &FieldElement,
        contract_type: &ContractType,
        block_number: u64,
    ) -> Result<(), StorageError> {
        info!(
            "Registering contract info {:?} for contract {}",
            contract_type, contract_address
        );

        let pk = format!("COLLECTION#{}", to_hex_str(contract_address));
        let sk = "COLLECTION".to_string();

        // Construct the data map for the contract
        let mut data = HashMap::new();
        data.insert(
            "ContractAddress".to_string(),
            AttributeValue::S(to_hex_str(contract_address)),
        );
        data.insert(
            "ContractType".to_string(),
            AttributeValue::S(contract_type.to_string()),
        );
        data.insert(
            "BlockNumber".to_string(),
            AttributeValue::N(block_number.to_string()),
        );

        // Try to create the contract info with a condition that the PK should not already exist
        let put_item_output = self
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(pk))
            .item("SK", AttributeValue::S(sk))
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_number)),
            )
            .item(
                "GSI4SK".to_string(),
                AttributeValue::S(format!("COLLECTION#{}", contract_address)),
            )
            .item("Data", AttributeValue::M(data))
            .item(
                "Type",
                AttributeValue::S(EntityType::Collection.to_string()),
            )
            .condition_expression("attribute_not_exists(PK)")
            .send()
            .await;

        match put_item_output {
            Ok(_) => Ok(()),
            Err(e) => {
                // If the condition failed, it means the contract info already exists no need to create it
                error!("Collection already exist: {:?}", e);
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn set_block_info(&self, block_number: u64, info: BlockInfo) -> Result<(), StorageError> {
        info!("Setting block info {:?} for block #{}", info, block_number);

        let pk = format!("BLOCK#{}", block_number);
        let sk = "BLOCK".to_string();

        // Construct the data map for the block
        let mut data = HashMap::new();
        data.insert(
            "IndexerVersion".to_string(),
            AttributeValue::S(info.indexer_version.to_string()),
        );
        data.insert(
            "IndexerIdentifier".to_string(),
            AttributeValue::S(info.indexer_identifier),
        );
        data.insert(
            "Status".to_string(),
            AttributeValue::S(info.status.to_string()),
        );

        // Upsert the block info
        let put_item_output = self
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(pk))
            .item("SK", AttributeValue::S(sk))
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_number)),
            )
            .item("GSI4SK".to_string(), AttributeValue::S("BLOCK".to_string()))
            .item("Data", AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Block.to_string()))
            .send()
            .await;

        match put_item_output {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("DynamoDB error: {:?}", e);
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn get_block_info(&self, block_number: u64) -> Result<BlockInfo, StorageError> {
        info!("Getting block info for block #{}", block_number);

        let pk = format!("BLOCK#{}", block_number);
        let sk = "BLOCK".to_string();

        // Query DynamoDB for the block info
        let get_item_output = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key("PK", AttributeValue::S(pk))
            .key("SK", AttributeValue::S(sk))
            .send()
            .await;

        info!("get_item_output: {:?}", get_item_output);
        match get_item_output {
            Ok(output) => {
                if let Some(item) = output.item {
                    if let Some(AttributeValue::M(data)) = item.get("Data") {
                        let indexer_version_str =
                            if let Some(AttributeValue::S(value)) = data.get("indexer_version") {
                                value.clone()
                            } else {
                                return Err(StorageError::DatabaseError);
                            };

                        let indexer_version = indexer_version_str
                            .parse()
                            .map_err(|_| StorageError::DatabaseError)?;

                        let indexer_identifier = if let Some(AttributeValue::S(value)) =
                            data.get("indexer_identifier")
                        {
                            value.clone()
                        } else {
                            return Err(StorageError::DatabaseError);
                        };

                        let status_str = if let Some(AttributeValue::S(value)) = data.get("status")
                        {
                            value.clone()
                        } else {
                            return Err(StorageError::DatabaseError);
                        };

                        let status: BlockIndexingStatus = status_str
                            .parse()
                            .map_err(|_| StorageError::InvalidStatus)?;

                        Ok(BlockInfo {
                            indexer_version,
                            indexer_identifier,
                            status,
                        })
                    } else {
                        error!("Data NotFound error");
                        Err(StorageError::NotFound)
                    }
                } else {
                    error!("Item NotFound error");
                    Err(StorageError::NotFound)
                }
            }
            Err(e) => {
                error!("Table NotFound error: {:?}", e);
                Err(StorageError::DatabaseError)
            }
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

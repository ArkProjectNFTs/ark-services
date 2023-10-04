use anyhow::{anyhow, Result};
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
use aws_sdk_dynamodb::{types::AttributeValue, types::ReturnValue, Client};
use chrono::Utc;
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, error, info, trace};

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
    ) -> Result<()>;
    async fn update_indexer_progress(&self, task_id: String, value: f64) -> Result<()>;
}

#[async_trait]
impl AWSDynamoStorage for DynamoStorage {
    async fn update_indexer_task_status(
        &self,
        task_id: String,
        indexer_version: String,
        status: IndexerStatus,
    ) -> Result<()> {
        let now = Utc::now();
        let unix_timestamp = now.timestamp();

        let status_string = match status {
            IndexerStatus::Running => "running",
            IndexerStatus::Stopped => "stopped",
        }
        .to_string();

        trace!("Updating table {}...", self.table_name);

        let response = self
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK", AttributeValue::S(String::from("INDEXER")))
            .item("SK", AttributeValue::S(format!("TASK#{}", task_id)))
            .item("status", AttributeValue::S(status.to_string()))
            .item("last_update", AttributeValue::N(unix_timestamp.to_string()))
            .item("version", AttributeValue::S(indexer_version.to_string()))
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
        token: &TokenFromEvent,
        block_number: u64,
    ) -> Result<(), StorageError> {
        debug!("Registering mint {:?}", token);

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
                    AttributeValue::N(token.formated_token_id.token_id.to_string()),
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
                    AttributeValue::N(token.formated_token_id.token_id.to_string()),
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
        debug!("Registering event {:?}", event);

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
                println!("event: {:?}", event);
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
                    "TokenID".to_string(),
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
                    "EventID".to_string(),
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
        debug!("Getting contract info for contract {}", contract_address);

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
        debug!(
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
                info!("Collection already exist: {:?}", e);
                Err(StorageError::DatabaseError)
            }
        }
    }

    async fn set_block_info(&self, block_number: u64, info: BlockInfo) -> Result<(), StorageError> {
        debug!("Setting block info {:?} for block #{}", info, block_number);

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
        debug!("Getting block info for block #{}", block_number);

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

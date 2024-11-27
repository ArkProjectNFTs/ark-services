use super::ArkEventProvider;
use crate::{convert, DynamoDbCtx, DynamoDbOutput, EntityType, ProviderError};
use arkproject::pontos::storage::types::{
    EventType, TokenEvent, TokenSaleEvent, TokenTransferEvent,
};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnConsumedCapacity};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, info, warn};

/// DynamoDB provider for events.
pub struct DynamoDbEventProvider {
    table_name: String,
    key_prefix: String,
    limit: Option<i32>,
}

impl DynamoDbEventProvider {
    pub fn new(table_name: &str, limit: Option<i32>) -> Self {
        DynamoDbEventProvider {
            table_name: table_name.to_string(),
            key_prefix: "EVENT".to_string(),
            limit,
        }
    }

    fn get_pk(&self, contract_address: &str, event_id: &str) -> String {
        format!("{}#{}#{}", self.key_prefix, contract_address, event_id)
    }

    fn get_sk(&self, event_type: &EventType) -> String {
        format!("{}#{}", self.key_prefix, event_type)
    }

    pub fn data_to_sale_event(
        data: &HashMap<String, AttributeValue>,
    ) -> Result<TokenSaleEvent, ProviderError> {
        let block_number = match convert::attr_to_u64(data, "BlockNumber") {
            Ok(bn) => Some(bn),
            Err(_) => None,
        };

        let updated_at = match convert::attr_to_u64(data, "UpdatedAt") {
            Ok(u) => Some(u),
            Err(_) => None,
        };

        let event_type_str = &convert::attr_to_str(data, "EventType")?;
        match EventType::from_str(event_type_str.as_str()) {
            Ok(event_type) => {
                let currency_address = match convert::attr_to_str(data, "CurrencyContractAddress") {
                    Ok(ca) => Some(ca),
                    Err(_) => None,
                };

                Ok(TokenSaleEvent {
                    chain_id: convert::attr_to_str(data, "ChainID")?,
                    event_id: convert::attr_to_str(data, "EventId")?,
                    event_type,
                    timestamp: convert::attr_to_u64(data, "Timestamp")?,
                    from_address: convert::attr_to_str(data, "FromAddress")?,
                    to_address: convert::attr_to_str(data, "ToAddress")?,
                    nft_contract_address: convert::attr_to_str(data, "NftContractAddress")?,
                    nft_type: convert::attr_to_str(data, "NftType").ok(),
                    token_id: convert::attr_to_str(data, "TokenId")?,
                    token_id_hex: convert::attr_to_str(data, "TokenIdHex")?,
                    transaction_hash: convert::attr_to_str(data, "TransactionHash")?,
                    block_number,
                    updated_at,
                    currency_address,
                    marketplace_contract_address: convert::attr_to_str(
                        data,
                        "MarketplaceContractAddress",
                    )?,
                    marketplace_name: convert::attr_to_str(data, "MarketplaceName")?,
                    price: convert::attr_to_str(data, "Price")?,
                    quantity: convert::attr_to_u64(data, "Quantity")?,
                    chain_id: "0x534e5f4d41494e".to_string(),
                })
            }
            Err(_) => Err(ProviderError::ParsingError(
                "EventType is unknown".to_string(),
            )),
        }
    }

    pub fn data_to_transfer_event(
        data: &HashMap<String, AttributeValue>,
    ) -> Result<TokenTransferEvent, ProviderError> {
        let block_number = match convert::attr_to_u64(data, "BlockNumber") {
            Ok(bn) => Some(bn),
            Err(_) => None,
        };

        let updated_at = match convert::attr_to_u64(data, "UpdatedAt") {
            Ok(u) => Some(u),
            Err(_) => None,
        };

        let event_type_str = &convert::attr_to_str(data, "EventType")?;
        match EventType::from_str(event_type_str.as_str()) {
            Ok(event_type) => Ok(TokenTransferEvent {
                event_id: convert::attr_to_str(data, "EventId")?,
                event_type,
                timestamp: convert::attr_to_u64(data, "Timestamp")?,
                from_address: convert::attr_to_str(data, "FromAddress")?,
                to_address: convert::attr_to_str(data, "ToAddress")?,
                contract_address: convert::attr_to_str(data, "ContractAddress")?,
                contract_type: convert::attr_to_str(data, "ContractType")?,
                token_id: convert::attr_to_str(data, "TokenId")?,
                token_id_hex: convert::attr_to_str(data, "TokenIdHex")?,
                transaction_hash: convert::attr_to_str(data, "TransactionHash")?,
                block_number,
                updated_at,
                ..Default::default()
            }),
            Err(_) => Err(ProviderError::ParsingError(
                "EventType is unknown".to_string(),
            )),
        }
    }

    pub fn sale_event_to_data(event: &TokenSaleEvent) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            "Timestamp".to_string(),
            AttributeValue::N(event.timestamp.to_string()),
        );

        map.insert(
            "Quantity".to_string(),
            AttributeValue::N(event.quantity.to_string()),
        );

        map.insert(
            "Price".to_string(),
            AttributeValue::S(event.price.to_string()),
        );

        if let Some(currency_address) = event.currency_address.clone() {
            map.insert(
                "CurrencyContractAddress".to_string(),
                AttributeValue::S(currency_address.to_string()),
            );
        }

        map.insert(
            "MarketplaceName".to_string(),
            AttributeValue::S(event.marketplace_name.to_string()),
        );

        map.insert(
            "MarketplaceContractAddress".to_string(),
            AttributeValue::S(event.marketplace_contract_address.to_string()),
        );

        map.insert(
            "FromAddress".to_string(),
            AttributeValue::S(event.from_address.clone()),
        );
        map.insert(
            "ToAddress".to_string(),
            AttributeValue::S(event.to_address.clone()),
        );
        map.insert(
            "NftContractAddress".to_string(),
            AttributeValue::S(event.nft_contract_address.clone()),
        );

        if let Some(nft_type) = event.nft_type.clone() {
            map.insert("NftType".to_string(), AttributeValue::S(nft_type));
        }

        map.insert(
            "TransactionHash".to_string(),
            AttributeValue::S(event.transaction_hash.clone()),
        );
        map.insert(
            "TokenId".to_string(),
            AttributeValue::S(event.token_id.clone()),
        );
        map.insert(
            "TokenIdHex".to_string(),
            AttributeValue::S(event.token_id_hex.clone()),
        );
        map.insert(
            "EventType".to_string(),
            AttributeValue::S(event.event_type.clone().to_string()),
        );
        map.insert(
            "EventId".to_string(),
            AttributeValue::S(event.event_id.clone()),
        );

        if let Some(block_number) = event.block_number {
            map.insert(
                "BlockNumber".to_string(),
                AttributeValue::N(block_number.to_string()),
            );
        }

        if let Some(updated_at) = event.updated_at {
            map.insert(
                "UpdatedAt".to_string(),
                AttributeValue::N(updated_at.to_string()),
            );
        }

        map
    }

    pub fn transfer_event_to_data(event: &TokenTransferEvent) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            "Timestamp".to_string(),
            AttributeValue::N(event.timestamp.to_string()),
        );
        map.insert(
            "FromAddress".to_string(),
            AttributeValue::S(event.from_address.clone()),
        );
        map.insert(
            "ToAddress".to_string(),
            AttributeValue::S(event.to_address.clone()),
        );
        map.insert(
            "ContractAddress".to_string(),
            AttributeValue::S(event.contract_address.clone()),
        );
        map.insert(
            "TransactionHash".to_string(),
            AttributeValue::S(event.transaction_hash.clone()),
        );
        map.insert(
            "TokenId".to_string(),
            AttributeValue::S(event.token_id.clone()),
        );
        map.insert(
            "TokenIdHex".to_string(),
            AttributeValue::S(event.token_id_hex.clone()),
        );
        map.insert(
            "ContractType".to_string(),
            AttributeValue::S(event.contract_type.clone().to_string()),
        );
        map.insert(
            "EventType".to_string(),
            AttributeValue::S(event.event_type.clone().to_string()),
        );
        map.insert(
            "EventId".to_string(),
            AttributeValue::S(event.event_id.clone()),
        );

        if let Some(block_number) = event.block_number {
            map.insert(
                "BlockNumber".to_string(),
                AttributeValue::N(block_number.to_string()),
            );
        }

        if let Some(updated_at) = event.updated_at {
            map.insert(
                "UpdatedAt".to_string(),
                AttributeValue::N(updated_at.to_string()),
            );
        }

        map
    }
}

#[async_trait]
impl ArkEventProvider for DynamoDbEventProvider {
    type Client = DynamoClient;

    async fn register_sale_event(
        &self,
        ctx: &DynamoDbCtx,
        event: &TokenSaleEvent,
        block_timestamp: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let data = Self::sale_event_to_data(event);

        if event.nft_type.is_none() {
            return Err(ProviderError::MissingDataError(
                "NFT type is empty".to_string(),
            ));
        }

        let pk = event.nft_type.clone().unwrap();
        let pk_value = self.get_pk(pk.as_str(), &event.event_id);

        info!("Registering sale event with PK: {}", pk_value);

        let _r = ctx
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK".to_string(), AttributeValue::S(pk_value))
            .item("SK".to_string(), AttributeValue::S("EVENT".to_string()))
            .item("Type".to_string(), AttributeValue::S("Event".to_string()))
            .item(
                "GSI1PK".to_string(),
                AttributeValue::S(format!("CONTRACT#{}", event.nft_contract_address)),
            )
            .item(
                "GSI1SK".to_string(),
                AttributeValue::S(format!("EVENT#{}", event.timestamp)),
            )
            .item(
                "GSI2PK".to_string(),
                AttributeValue::S(format!(
                    "TOKEN#{}#{}",
                    event.nft_contract_address, event.token_id_hex,
                )),
            )
            .item(
                "GSI2SK".to_string(),
                AttributeValue::S(format!("EVENT#{}", event.event_id)),
            )
            .item(
                "GSI3PK".to_string(),
                AttributeValue::S(format!("EVENT_FROM#{}", event.from_address)),
            )
            .item(
                "GSI3SK".to_string(),
                AttributeValue::S(format!("TIMESTAMP#{}", block_timestamp)),
            )
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_timestamp)),
            )
            .item(
                "GSI4SK".to_string(),
                AttributeValue::S(self.get_pk(&event.nft_contract_address, &event.event_id)),
            )
            .item(
                "GSI5PK".to_string(),
                AttributeValue::S(format!("EVENT_TO#{}", event.to_address)),
            )
            .item(
                "GSI5SK".to_string(),
                AttributeValue::S(format!("TIMESTAMP#{}", block_timestamp)),
            )
            .item("GSI6PK".to_string(), AttributeValue::S("EVENT".to_string()))
            .item(
                "GSI6SK".to_string(),
                AttributeValue::N(block_timestamp.to_string()),
            )
            .item(
                "GSI7PK".to_string(),
                AttributeValue::S(format!(
                    "{}#{}",
                    event.event_type,
                    event.marketplace_name.to_uppercase()
                )),
            )
            .item(
                "GSI7SK".to_string(),
                AttributeValue::N(block_timestamp.to_string()),
            )
            .item("Data".to_string(), AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Event.to_string()))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        Ok(().into())
    }

    async fn register_transfer_event(
        &self,
        ctx: &DynamoDbCtx,
        event: &TokenTransferEvent,
        block_timestamp: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let data = Self::transfer_event_to_data(event);
        let pk = self.get_pk(&event.contract_address, &event.event_id);

        info!("Registering transfer event with PK: {}", pk);

        let _r = ctx
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item("PK".to_string(), AttributeValue::S(pk))
            .item(
                "SK".to_string(),
                AttributeValue::S(self.get_sk(&event.event_type)),
            )
            .item("Type".to_string(), AttributeValue::S("Event".to_string()))
            .item(
                "GSI1PK".to_string(),
                AttributeValue::S(format!("CONTRACT#{}", event.contract_address)),
            )
            .item(
                "GSI1SK".to_string(),
                AttributeValue::S(format!("EVENT#{}", event.timestamp)),
            )
            .item(
                "GSI2PK".to_string(),
                AttributeValue::S(format!(
                    "TOKEN#{}#{}",
                    event.contract_address, event.token_id_hex,
                )),
            )
            .item(
                "GSI2SK".to_string(),
                AttributeValue::S(format!("EVENT#{}", event.event_id)),
            )
            .item(
                "GSI3PK".to_string(),
                AttributeValue::S(format!("EVENT_FROM#{}", event.from_address)),
            )
            .item(
                "GSI3SK".to_string(),
                AttributeValue::S(format!("TIMESTAMP#{}", block_timestamp)),
            )
            .item(
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_timestamp)),
            )
            .item(
                "GSI4SK".to_string(),
                AttributeValue::S(self.get_pk(&event.contract_address, &event.event_id)),
            )
            .item(
                "GSI5PK".to_string(),
                AttributeValue::S(format!("EVENT_TO#{}", event.to_address)),
            )
            .item(
                "GSI5SK".to_string(),
                AttributeValue::S(format!("TIMESTAMP#{}", block_timestamp)),
            )
            .item("GSI6PK".to_string(), AttributeValue::S("EVENT".to_string()))
            .item(
                "GSI6SK".to_string(),
                AttributeValue::N(block_timestamp.to_string()),
            )
            .item("Data".to_string(), AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Event.to_string()))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;
        Ok(().into())
    }

    async fn get_event(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        event_id: &str,
    ) -> Result<DynamoDbOutput<Option<TokenEvent>>, ProviderError> {
        // Streamlined key creation
        let key = {
            let mut key = HashMap::new();
            key.insert(
                "PK".to_string(),
                AttributeValue::S(self.get_pk(contract_address, event_id)),
            );
            key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));
            key
        };

        let r = ctx
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let consumed_capacity_units = r.consumed_capacity().and_then(|c| c.capacity_units);

        if let Some(item) = &r.item {
            let data = convert::attr_to_map(item, "Data")?;

            if let Some(event_type) = data.get("EventType") {
                let event_type = event_type.as_s().unwrap();
                if event_type == "SALE" {
                    let data = Self::data_to_sale_event(&data)?;
                    let result = Some(TokenEvent::Sale(data));
                    return Ok(DynamoDbOutput::new(result, consumed_capacity_units, None));
                } else {
                    let data: TokenTransferEvent = Self::data_to_transfer_event(&data)?;
                    return Ok(DynamoDbOutput::new(
                        Some(TokenEvent::Transfer(data)),
                        consumed_capacity_units,
                        None,
                    ));
                }
            }
        }
        Ok(DynamoDbOutput::new(None, consumed_capacity_units, None))
    }

    async fn get_token_events(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_hex_id: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenEvent>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(":event".to_string(), AttributeValue::S("EVENT".to_string()));
        values.insert(
            ":token".to_string(),
            AttributeValue::S(format!("TOKEN#{}#{}", contract_address, token_hex_id)),
        );

        let r = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some(
                "GSI2PK = :token AND begins_with(GSI2SK, :event)".to_string(),
            ))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .set_expression_attribute_values(Some(values))
            .set_limit(self.limit)
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        debug!("Query result. Items: {:?}", r.items);

        let mut res = vec![];
        if let Some(items) = r.clone().items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                if let Some(event_type) = data.get("EventType") {
                    let event_type = event_type.as_s().unwrap();
                    if event_type == "SALE" {
                        res.push(TokenEvent::Sale(Self::data_to_sale_event(&data)?));
                    } else {
                        res.push(TokenEvent::Transfer(Self::data_to_transfer_event(&data)?));
                    }
                }
            }
        }

        let consumed_capacity_units = match r.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        Ok(DynamoDbOutput::new_lek(
            res,
            consumed_capacity_units,
            r.last_evaluated_key,
            None,
        ))
    }

    async fn get_owner_from_events(
        &self,
        ctx: &DynamoDbCtx,
        owner_address: &str,
        cursor_name: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenEvent>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(":event".to_string(), AttributeValue::S("EVENT".to_string()));
        values.insert(
            ":owner".to_string(),
            AttributeValue::S(format!("EVENT_FROM#{}", owner_address)),
        );

        let lek = ctx
            .multiple_exclusive_start_keys
            .get(cursor_name)
            .unwrap_or(&None);

        let r = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI3PK-GSI3SK-index")
            .set_key_condition_expression(Some(
                "GSI3PK = :owner AND begins_with(GSI3SK, :event)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(lek.clone())
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = r.clone().items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                if let Some(event_type) = data.get("EventType") {
                    let event_type = event_type.as_s().unwrap();
                    if event_type == "SALE" {
                        res.push(TokenEvent::Sale(Self::data_to_sale_event(&data)?));
                    } else {
                        res.push(TokenEvent::Transfer(Self::data_to_transfer_event(&data)?));
                    }
                }
            }
        }

        let consumed_capacity_units = match r.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        Ok(DynamoDbOutput::new_lek(
            res,
            consumed_capacity_units,
            r.last_evaluated_key,
            None,
        ))
    }

    async fn get_owner_to_events(
        &self,
        ctx: &DynamoDbCtx,
        owner_address: &str,
        cursor_name: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenEvent>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(":event".to_string(), AttributeValue::S("EVENT".to_string()));
        values.insert(
            ":owner".to_string(),
            AttributeValue::S(format!("EVENT_TO#{}", owner_address)),
        );

        let lek = ctx
            .multiple_exclusive_start_keys
            .get(cursor_name)
            .unwrap_or(&None);

        let r = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI5PK-GSI5SK-index")
            .set_key_condition_expression(Some(
                "GSI5PK = :owner AND begins_with(GSI5SK, :event)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(lek.clone())
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = r.clone().items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                if let Some(event_type) = data.get("EventType") {
                    let event_type = event_type.as_s().unwrap();
                    if event_type == "SALE" {
                        res.push(TokenEvent::Sale(Self::data_to_sale_event(&data)?));
                    } else {
                        res.push(TokenEvent::Transfer(Self::data_to_transfer_event(&data)?));
                    }
                }
            }
        }

        let consumed_capacity_units = match r.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        Ok(DynamoDbOutput::new_lek(
            res,
            consumed_capacity_units,
            r.last_evaluated_key,
            None,
        ))
    }

    async fn get_events(
        &self,
        ctx: &DynamoDbCtx,
    ) -> Result<DynamoDbOutput<Vec<TokenEvent>>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(":event".to_string(), AttributeValue::S("EVENT".to_string()));

        let r: aws_sdk_dynamodb::operation::query::QueryOutput = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI6PK-GSI6SK-index")
            .set_key_condition_expression(Some("GSI6PK = :event".to_string()))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .set_limit(self.limit)
            .scan_index_forward(false)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];

        info!("Query result items: {:?}", r.items);

        if let Some(items) = r.clone().items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                if let Some(event_type) = data.get("EventType") {
                    let event_type = event_type.as_s().unwrap();
                    if event_type == "SALE" {
                        res.push(TokenEvent::Sale(Self::data_to_sale_event(&data)?));
                    } else {
                        res.push(TokenEvent::Transfer(Self::data_to_transfer_event(&data)?));
                    }
                }
            }
        }

        let consumed_capacity_units = match r.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        Ok(DynamoDbOutput::new_lek(
            res,
            consumed_capacity_units,
            r.last_evaluated_key,
            None,
        ))
    }

    async fn get_contract_events(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenEvent>>, ProviderError> {
        let gsi1pk_value = format!("CONTRACT#{}", contract_address);
        info!("GSI1PK value: {}", gsi1pk_value);

        let mut values = HashMap::new();
        values.insert(":event".to_string(), AttributeValue::S("EVENT".to_string()));
        values.insert(":contract".to_string(), AttributeValue::S(gsi1pk_value));

        let query_output = ctx
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI1PK-GSI1SK-index")
            .set_key_condition_expression(Some(
                "GSI1PK = :contract AND begins_with(GSI1SK, :event)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .set_exclusive_start_key(ctx.exclusive_start_key.clone())
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .set_limit(self.limit)
            .scan_index_forward(false)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = query_output.clone().items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                if let Some(event_type_av) = data.get("EventType") {
                    if let Ok(event_type) = event_type_av.as_s() {
                        match event_type.as_str() {
                            "SALE" => {
                                let result = Self::data_to_sale_event(&data)?;
                                res.push(TokenEvent::Sale(result));
                            }
                            "TRANSFER" => {
                                let result = Self::data_to_transfer_event(&data)?;
                                res.push(TokenEvent::Transfer(result));
                            }
                            "MINT" => {
                                let result = Self::data_to_transfer_event(&data)?;
                                res.push(TokenEvent::Transfer(result));
                            }
                            _ => {
                                warn!("Unknown event type: {}", event_type);
                            }
                        };
                    }
                }
            }
        }

        let consumed_capacity_units = match query_output.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        Ok(DynamoDbOutput::new_lek(
            res,
            consumed_capacity_units,
            query_output.last_evaluated_key,
            None,
        ))
    }
}

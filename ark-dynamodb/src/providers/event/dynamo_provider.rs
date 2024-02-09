use arkproject::pontos::storage::types::{EventType, TokenEvent};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnConsumedCapacity};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

use super::ArkEventProvider;
use crate::{convert, DynamoDbCtx, DynamoDbOutput, EntityType, ProviderError};

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

    fn get_sk(&self) -> String {
        self.key_prefix.clone()
    }

    pub fn data_to_event(
        data: &HashMap<String, AttributeValue>,
    ) -> Result<TokenEvent, ProviderError> {
        Ok(TokenEvent {
            event_id: convert::attr_to_str(data, "EventId")?,
            event_type: EventType::from_str(&convert::attr_to_str(data, "EventType")?).unwrap(),
            timestamp: convert::attr_to_u64(data, "Timestamp")?,
            from_address: convert::attr_to_str(data, "FromAddress")?,
            to_address: convert::attr_to_str(data, "ToAddress")?,
            contract_address: convert::attr_to_str(data, "ContractAddress")?,
            contract_type: convert::attr_to_str(data, "ContractType")?,
            token_id: convert::attr_to_str(data, "TokenId")?,
            token_id_hex: convert::attr_to_str(data, "TokenIdHex")?,
            transaction_hash: convert::attr_to_str(data, "TransactionHash")?,
            block_number: convert::attr_to_u64(data, "BlockNumber").ok(),
            updated_at: convert::attr_to_u64(data, "UpdatedAt").ok(),
        })
    }

    pub fn event_to_data(event: &TokenEvent) -> HashMap<String, AttributeValue> {
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

    async fn register_event(
        &self,
        ctx: &DynamoDbCtx,
        event: &TokenEvent,
        block_timestamp: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError> {
        let data = Self::event_to_data(event);

        let _r = ctx
            .client
            .put_item()
            .table_name(self.table_name.clone())
            .item(
                "PK".to_string(),
                AttributeValue::S(self.get_pk(&event.contract_type, &event.event_id)),
            )
            .item("SK".to_string(), AttributeValue::S(self.get_sk()))
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
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(contract_address, event_id)),
        );
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let r = ctx
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .return_consumed_capacity(ReturnConsumedCapacity::Total)
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let consumed_capacity_units = match r.consumed_capacity() {
            Some(c) => c.capacity_units,
            None => None,
        };

        if let Some(item) = &r.item {
            let data = convert::attr_to_map(item, "Data")?;
            Ok(DynamoDbOutput::new(
                Some(Self::data_to_event(&data)?),
                consumed_capacity_units,
                None,
            ))
        } else {
            Ok(DynamoDbOutput::new(None, consumed_capacity_units, None))
        }
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

        let mut res = vec![];
        if let Some(items) = r.clone().items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(Self::data_to_event(&data)?);
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
                res.push(Self::data_to_event(&data)?);
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
                res.push(Self::data_to_event(&data)?);
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
                res.push(Self::data_to_event(&data)?);
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
                res.push(Self::data_to_event(&data)?);
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

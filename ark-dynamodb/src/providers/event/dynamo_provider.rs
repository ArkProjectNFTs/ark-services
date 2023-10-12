use arkproject::pontos::storage::types::{EventType, TokenEvent};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::str::FromStr;

use super::ArkEventProvider;
use crate::{convert, EntityType, ProviderError};

/// DynamoDB provider for events.
pub struct DynamoDbEventProvider {
    table_name: String,
    key_prefix: String,
}

impl DynamoDbEventProvider {
    pub fn new(table_name: &str) -> Self {
        DynamoDbEventProvider {
            table_name: table_name.to_string(),
            key_prefix: "EVENT".to_string(),
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

        map
    }
}

#[async_trait]
impl ArkEventProvider for DynamoDbEventProvider {
    type Client = DynamoClient;

    async fn register_event(
        &self,
        client: &Self::Client,
        event: &TokenEvent,
        block_number: u64,
    ) -> Result<(), ProviderError> {
        let data = Self::event_to_data(event);

        let put_item_output = client
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
                AttributeValue::S(format!("EVENT#{}", event.event_id)),
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
                "GSI4PK".to_string(),
                AttributeValue::S(format!("BLOCK#{}", block_number)),
            )
            .item(
                "GSI4SK".to_string(),
                AttributeValue::S(self.get_pk(&event.contract_address, &event.event_id)),
            )
            .item("Data".to_string(), AttributeValue::M(data))
            .item("Type", AttributeValue::S(EntityType::Event.to_string()))
            .return_values(ReturnValue::AllOld)
            .send()
            .await;

        put_item_output.map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_event(
        &self,
        client: &Self::Client,
        contract_address: &str,
        event_id: &str,
    ) -> Result<Option<TokenEvent>, ProviderError> {
        let mut key = HashMap::new();
        key.insert(
            "PK".to_string(),
            AttributeValue::S(self.get_pk(contract_address, event_id)),
        );
        key.insert("SK".to_string(), AttributeValue::S(self.key_prefix.clone()));

        let req = client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        if let Some(item) = &req.item {
            let data = convert::attr_to_map(item, "Data")?;
            Ok(Some(Self::data_to_event(&data)?))
        } else {
            Ok(None)
        }
    }

    async fn get_token_events(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_hex_id: &str,
    ) -> Result<Vec<TokenEvent>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(":event".to_string(), AttributeValue::S("EVENT".to_string()));
        values.insert(
            ":token".to_string(),
            AttributeValue::S(format!("TOKEN#{}#{}", contract_address, token_hex_id)),
        );

        let req = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI2PK-GSI2SK-index")
            .set_key_condition_expression(Some(
                "GSI2PK = :token AND begins_with(GSI2SK, :event)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = req.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(Self::data_to_event(&data)?);
            }
        }

        Ok(res)
    }

    async fn get_contract_events(
        &self,
        client: &Self::Client,
        contract_address: &str,
    ) -> Result<Vec<TokenEvent>, ProviderError> {
        let mut values = HashMap::new();
        values.insert(":event".to_string(), AttributeValue::S("EVENT".to_string()));
        values.insert(
            ":contract".to_string(),
            AttributeValue::S(format!("CONTRACT#{}", contract_address)),
        );

        let req = client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI1PK-GSI1SK-index")
            .set_key_condition_expression(Some(
                "GSI1PK = :contract AND begins_with(GSI1SK, :event)".to_string(),
            ))
            .set_expression_attribute_values(Some(values))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;

        let mut res = vec![];
        if let Some(items) = req.items {
            for i in items {
                let data = convert::attr_to_map(&i, "Data")?;
                res.push(Self::data_to_event(&data)?);
            }
        }

        Ok(res)
    }
}

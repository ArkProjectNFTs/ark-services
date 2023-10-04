use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;

use super::{types::EventData, ArkEventProvider};
use crate::{convert, ProviderError};

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
}

#[async_trait]
impl ArkEventProvider for DynamoDbEventProvider {
    type Client = DynamoClient;

    async fn get_event(
        &self,
        client: &Self::Client,
        contract_address: &str,
        event_id: &str,
    ) -> Result<Option<EventData>, ProviderError> {
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

            Ok(Some(EventData {
                block_number: convert::attr_to_u64(&data, "BlockNumber")?,
                event_id: convert::attr_to_str(&data, "EventId")?,
                event_type: convert::attr_to_str(&data, "EventType")?,
                timestamp: convert::attr_to_u64(&data, "Timestamp")?,
                from_address: convert::attr_to_str(&data, "FromAddress")?,
                to_address: convert::attr_to_str(&data, "ToAddress")?,
                contract_address: convert::attr_to_str(&data, "ContractAddress")?,
                contract_type: convert::attr_to_str(&data, "ContractType")?,
                token_id: convert::attr_to_str(&data, "TokenId")?,
                transaction_hash: convert::attr_to_str(&data, "TransactionHash")?,
            }))
        } else {
            Ok(None)
        }
    }
}

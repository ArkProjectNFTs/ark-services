use aws_sdk_dynamodb::types::{AttributeValue, ConsumedCapacity, ReturnValue};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::ProviderError;

pub struct DynamoDbCapacityProvider;

// TODO: THIS IS NOT OPTI TO REGISTER THE CAPACITY ON EACH OPERATION.
// For now, we're exploring the capacity system. But we need to think
// to a better way to register using batches.

impl DynamoDbCapacityProvider {
    /// Register the capacity used for the given operation.
    pub async fn register_consumed_capacity(
        client: &DynamoClient,
        operation: &str,
        consumed_capacity: Option<ConsumedCapacity>,
    ) -> Result<(), ProviderError> {
        if consumed_capacity.is_none() {
            return Ok(());
        }

        let capacity = consumed_capacity
            .unwrap()
            .capacity_units
            .unwrap_or(0.0)
            .to_string();

        let mut items = HashMap::new();

        // TODO: PK must be something like the UserID or the APIKey, to ensure we can follow the consumption of each users.
        let random_uuid = Uuid::new_v4();
        let pk = random_uuid.to_hyphenated().to_string();

        let sk = format!("{}#{}", operation, now());

        items.insert("PK".to_string(), AttributeValue::S(pk));
        items.insert("SK".to_string(), AttributeValue::S(sk));
        items.insert("Capacity".to_string(), AttributeValue::N(capacity));

        let put_item_output = client
            .put_item()
            .table_name("ark_project_metrics_capacity")
            .set_item(Some(items))
            .return_values(ReturnValue::AllOld)
            .send()
            .await;

        put_item_output.map_err(|e| ProviderError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting time")
        .as_secs()
}

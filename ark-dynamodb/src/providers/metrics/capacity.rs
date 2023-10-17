use aws_sdk_dynamodb::types::{AttributeValue, ConsumedCapacity};
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
        consumed_capacity: &Option<ConsumedCapacity>,
    ) -> Result<(), ProviderError> {
        if consumed_capacity.is_none() {
            return Ok(());
        }

        let capacity = consumed_capacity
            .clone()
            .unwrap()
            .capacity_units
            .unwrap_or(0.0);

        Self::register_raw(client, operation, capacity).await
    }

    /// Register the capacity used for the given operation.
    pub async fn register_raw(
        client: &DynamoClient,
        operation: &str,
        capacity: f64,
    ) -> Result<(), ProviderError> {
        let mut items = HashMap::new();

        let random_uuid = Uuid::new_v4();
        let pk = random_uuid.to_hyphenated().to_string();
        let sk = operation.to_string();
        let ttl = get_ttl().to_string();

        items.insert("PK".to_string(), AttributeValue::S(pk));
        items.insert("SK".to_string(), AttributeValue::S(sk));
        items.insert("Ttl".to_string(), AttributeValue::N(ttl));
        items.insert(
            "Capacity".to_string(),
            AttributeValue::N(capacity.to_string()),
        );

        client
            .put_item()
            .table_name("ark_project_capacity")
            .set_item(Some(items))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

fn get_ttl() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting time");

    // 1 days ttl for statistics and cost estimation.
    (now + Duration::from_secs(24 * 60 * 60)).as_secs()
}

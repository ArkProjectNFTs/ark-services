use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ProviderError;

pub struct LambdaUsageProvider;

impl LambdaUsageProvider {
    /// Register the usage for a user for the given lambda.
    pub async fn register_usage(
        client: &DynamoClient,
        request_id: &str,
        api_key: &str,
        lambda_name: &str,
        capacity: f64,
        exec_time: u64,
    ) -> Result<(), ProviderError> {
        let mut items = HashMap::new();

        let pk = format!("REQ#{}", request_id);
        let sk = lambda_name.to_string();
        let gsi1pk = format!("APIKEY#{}", api_key);
        let gsi1sk = now().to_string();

        items.insert("PK".to_string(), AttributeValue::S(pk));
        items.insert("SK".to_string(), AttributeValue::S(sk));
        items.insert("GSI1PK".to_string(), AttributeValue::S(gsi1pk));
        items.insert("GSI1SK".to_string(), AttributeValue::N(gsi1sk));
        items.insert(
            "Capacity".to_string(),
            AttributeValue::N(capacity.to_string()),
        );
        items.insert(
            "ExecTime".to_string(),
            AttributeValue::N(exec_time.to_string()),
        );

        client
            .put_item()
            .table_name("ark_project_lambda_usage")
            .set_item(Some(items))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting time")
        .as_secs()
}

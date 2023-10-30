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
        exec_time: u128,
        response_status: String,
    ) -> Result<(), ProviderError> {
        let mut items = HashMap::new();

        let now = now().to_string();

        let pk = format!("REQ#{}", request_id);
        // TODO: something better here for the SK?
        let sk = lambda_name.to_string();

        let gsi1pk = format!("APIKEY#{}", api_key);
        let gsi1sk = now.clone();

        let lambda_name = lambda_name.to_string();

        items.insert("PK".to_string(), AttributeValue::S(pk));
        items.insert("SK".to_string(), AttributeValue::S(sk));
        items.insert("GSI1PK".to_string(), AttributeValue::S(gsi1pk));
        items.insert("GSI1SK".to_string(), AttributeValue::N(gsi1sk));
        items.insert(
            "Capacity".to_string(),
            AttributeValue::N(capacity.to_string()),
        );
        items.insert(
            "ExecTimeNano".to_string(),
            AttributeValue::N(exec_time.to_string()),
        );
        items.insert("LambdaName".to_string(), AttributeValue::S(lambda_name));
        items.insert("Timestamp".to_string(), AttributeValue::N(now));
        items.insert(
            "ResponseStatus".to_string(),
            AttributeValue::S(response_status),
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

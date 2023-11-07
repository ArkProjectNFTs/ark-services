use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ProviderError;

#[derive(Debug, Clone)]
pub struct LambdaUsageData {
    pub request_id: String,
    pub api_key: String,
    pub lambda_name: String,
    pub capacity: f64,
    pub exec_time: u128,
    pub response_status: i32,
    pub params: HashMap<String, String>,
}

pub struct LambdaUsageProvider;

impl LambdaUsageProvider {
    /// Register the usage for a user for the given lambda.
    pub async fn register_usage(
        client: &DynamoClient,
        usage_table_name: &str,
        data: &LambdaUsageData,
    ) -> Result<(), ProviderError> {
        let mut items = HashMap::new();

        let now = now().to_string();

        let pk = format!("REQ#{}", data.request_id);
        // TODO: something better here for the SK?
        let sk = data.lambda_name.clone();

        let gsi1pk = format!("APIKEY#{}", data.api_key.clone());
        let gsi1sk = now.clone();

        let gsi2pk = "REQ".to_string();
        let gsi2sk = now.clone();

        let mut params: HashMap<String, AttributeValue> = HashMap::new();
        for (k, v) in &data.params {
            params.insert(k.clone(), AttributeValue::S(v.clone()));
        }

        items.insert("PK".to_string(), AttributeValue::S(pk));
        items.insert("SK".to_string(), AttributeValue::S(sk));
        items.insert("GSI1PK".to_string(), AttributeValue::S(gsi1pk));
        items.insert("GSI1SK".to_string(), AttributeValue::N(gsi1sk));
        items.insert("GSI2PK".to_string(), AttributeValue::S(gsi2pk));
        items.insert("GSI2SK".to_string(), AttributeValue::N(gsi2sk));
        items.insert(
            "Capacity".to_string(),
            AttributeValue::N(data.capacity.to_string()),
        );
        items.insert(
            "ExecTimeMs".to_string(),
            AttributeValue::N(data.exec_time.to_string()),
        );
        items.insert(
            "LambdaName".to_string(),
            AttributeValue::S(data.lambda_name.clone()),
        );
        items.insert("Timestamp".to_string(), AttributeValue::N(now));
        items.insert(
            "ResponseStatus".to_string(),
            AttributeValue::N(data.response_status.to_string()),
        );
        items.insert("Params".to_string(), AttributeValue::M(params));
        items.insert(
            "ApiKey".to_string(),
            AttributeValue::S(data.api_key.clone()),
        );

        client
            .put_item()
            .table_name(usage_table_name)
            .set_item(Some(items))
            .send()
            .await
            .map_err(|e| ProviderError::DatabaseError(format!("{:?}", e)))?;
        Ok(())
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting time")
        .as_secs()
}

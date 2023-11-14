use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::trace;

use crate::providers::{ProviderError, SqlxCtx};

#[derive(Debug, Clone)]
pub struct LambdaUsageData {
    pub request_id: String,
    pub api_key: String,
    pub stage_name: String,
    pub lambda_name: String,
    pub capacity: f64,
    pub exec_time: u128,
    pub response_status: i32,
    pub params: HashMap<String, String>,
    pub http_method: String,
    pub http_path: String,
    pub source_ip: String,
}

impl LambdaUsageData {
    pub fn params_to_string(&self) -> String {
        let mut s = String::from("{");

        for (key, value) in &self.params {
            s.push_str(&format!("\"{}\": \"{}\"", key, value));
        }

        s.push('}');
        s
    }
}

pub struct LambdaUsageProvider;

impl LambdaUsageProvider {
    /// Register the usage for a user for the given lambda.
    pub async fn register_usage(
        client: &SqlxCtx,
        usage_table_name: &str,
        data: &LambdaUsageData,
    ) -> Result<(), ProviderError> {
        trace!("Registering usage {:?}", data);

        let q = format!("INSERT INTO {usage_table_name} (request_id, api_key, timestamp, capacity, execution_time_in_ms, response_status_code, request_method, request_path, request_params, ip, stage_name) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);");

        let _r = sqlx::query(&q)
            .bind(data.request_id.clone())
            .bind(data.api_key.clone())
            .bind(now() as i64)
            .bind(data.capacity)
            .bind(data.exec_time as i64)
            .bind(data.response_status)
            .bind(data.http_method.clone())
            .bind(data.http_path.clone())
            .bind(data.params_to_string())
            .bind(data.source_ip.clone())
            .bind(data.stage_name.clone())
            .execute(&client.pool)
            .await?;

        Ok(())
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting time")
        .as_secs()
}

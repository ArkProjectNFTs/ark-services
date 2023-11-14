//! Simple example of sqlx usage.
//!
//! To run this example, you can:
//! 1. start a docker with psql locally or use a know psql url.
//! 2. run `sqlx database reset/setup`.
//! 3. run `cargo run --examples usage`.
use ark_sqlx::providers::metrics::{LambdaUsageData, LambdaUsageProvider};
use ark_sqlx::providers::SqlxCtx;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let sqlx = SqlxCtx::new("postgresql://postgres:1234@localhost:9999/arksqlx")
        .await
        .expect("Can't connect to psql");

    let data = LambdaUsageData {
        request_id: "req_id".to_string(),
        api_key: "api_key".to_string(),
        lambda_name: "my_lambda".to_string(),
        http_method: "get".to_string(),
        http_path: "/path".to_string(),
        source_ip: "0.0.0.0".to_string(),
        capacity: 0.0,
        exec_time: 123,
        response_status: 200,
        params: HashMap::new(),
        stage_name: "local".to_string(),
    };

    match LambdaUsageProvider::register_usage(&sqlx, "lambda_usage", &data).await {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    }
}

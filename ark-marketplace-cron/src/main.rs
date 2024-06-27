pub mod models;
mod tasks;

use anyhow::Result;
use aws_config::BehaviorVersion;
use redis::aio::MultiplexedConnection;
use redis::Client;
use tasks::collections::{update_collections_floor, update_top_bid_collections};
use tasks::tokens::{cache_collection_pages, update_listed_tokens, update_top_bid_tokens};
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;

#[derive(Deserialize)]
struct DatabaseCredentials {
    username: String,
    password: String,
    dbname: String,
    port: u16,
    host: String,
}

async fn connect_redis() -> Result<MultiplexedConnection, Box<dyn Error>> {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL not set");
    let redis_username = std::env::var("REDIS_USERNAME").expect("REDIS_USERNAME not set");
    let redis_password = std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD not set");

    let client = Client::open(format!(
        "redis://{}:{}@{}",
        redis_username, redis_password, redis_url
    ))?;
    let connection = client.get_multiplexed_tokio_connection().await?;
    Ok(connection)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_logging();

    let database_url = get_database_url()
        .await
        .expect("Could not get the database URL");

    let db_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Could not connect to the database");

    match connect_redis().await {
        Ok(con) => {
            let _ = cache_collection_pages(&db_pool, con).await;
        }
        Err(e) => tracing::error!("Failed to connect to Redis: {}", e),
    }
    // @todo when adding new calculation add spawn & try_join!
    update_listed_tokens(&db_pool).await;
    update_top_bid_tokens(&db_pool).await;
    update_top_bid_collections(&db_pool).await;
    update_collections_floor(&db_pool).await;
    Ok(())
}

fn init_logging() {
    const DEFAULT_LOG_FILTER: &str = "trace";
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .or(EnvFilter::try_new(DEFAULT_LOG_FILTER))
                    .expect("Invalid RUST_LOG filters"),
            )
            .finish(),
    )
    .expect("Failed to set the global tracing subscriber");
}

async fn get_database_url() -> Result<String> {
    match std::env::var("DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(_) => {
            let secret_name = std::env::var("AWS_SECRET_NAME").expect("AWS_SECRET_NAME not set");
            let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
            let client = aws_sdk_secretsmanager::Client::new(&config);
            let secret_value = client
                .get_secret_value()
                .secret_id(secret_name)
                .send()
                .await?;
            let result = secret_value.secret_string.unwrap();

            let creds: DatabaseCredentials = serde_json::from_str(&result)?;
            let database_url = format!(
                "postgres://{}:{}@{}:{}/{}",
                creds.username, creds.password, creds.host, creds.port, creds.dbname
            );

            Ok(database_url)
        }
    }
}

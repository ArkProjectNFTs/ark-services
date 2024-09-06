pub mod models;
mod tasks;

use anyhow::Result;
use aws_config::BehaviorVersion;
use clap::{App, Arg};
use redis::aio::MultiplexedConnection;
use redis::Client;
use tasks::collections::{
    empty_floor_price, insert_floor_price, update_collections_market_data,
    update_top_bid_collections,
};
use tasks::tokens::{cache_collection_pages, update_listed_tokens, update_top_bid_tokens};
use tracing::info;
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
    info!("Starting marketplace cron job");
    let database_url = get_database_url()
        .await
        .expect("Could not get the database URL");

    let db_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Could not connect to the database");

    let matches = App::new("marketplace-cron")
        .arg(
            Arg::with_name("task")
                .long("task")
                .takes_value(true)
                .default_value("task_set1")
                .help("Sets the task set to run"),
        )
        .arg(
            Arg::with_name("disable-cache")
                .short('d')
                .long("disable-cache")
                .default_value("false")
                .takes_value(true)
                .help("Disables the cache if set to true"),
        )
        .get_matches();

    let task_set = matches.value_of("task").unwrap_or("");
    let should_cache_pages = !matches.is_present("disable-cache");

    match task_set {
        "task_set1" => match connect_redis().await {
            Ok(con) => {
                update_listed_tokens(&db_pool, con.clone()).await;
                update_top_bid_tokens(&db_pool, con.clone()).await;
                if should_cache_pages {
                    let _ = cache_collection_pages(&db_pool, con.clone()).await;
                }
            }
            Err(e) => tracing::error!("Failed to connect to Redis: {}", e),
        },
        "task_set2" => {
            update_top_bid_collections(&db_pool).await;
            update_collections_market_data(&db_pool).await;
            insert_floor_price(&db_pool).await;
        }
        "task_set3" => {
            empty_floor_price(&db_pool).await;
        }
        _ => {
            tracing::error!(
                "Invalid argument. Please use 'task_set1' or 'task_set2' or 'task_set3'"
            );
        }
    }
    tracing::info!("Marketplace cron job completed");
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

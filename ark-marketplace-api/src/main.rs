mod api_doc;

use actix_cors::Cors;
use actix_web::middleware::DefaultHeaders;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use ark_marketplace_api::routes::token;
use aws_config::BehaviorVersion;
use redis::{aio::MultiplexedConnection, Client};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use ark_marketplace_api::handlers::{
    collection_handler, default_handler, portfolio_handler, token_handler,
};

/// Initializes the logging, ensuring that the `RUST_LOG` environment
/// variable is always considered first.
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

#[derive(Deserialize)]
struct DatabaseCredentials {
    username: String,
    password: String,
    dbname: String,
    port: u16,
    host: String,
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

async fn get_write_database_url() -> Result<String> {
    match std::env::var("WRITE_DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(_) => {
            let secret_name = std::env::var("AWS_SECRET_NAME_WRITE_DB")
                .expect("AWS_SECRET_NAME_WRITE_DB not set");
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_logging();

    let database_url = get_database_url()
        .await
        .expect("Could not get the database URL");

    let write_database_url = get_write_database_url()
        .await
        .expect("Could not get the database URL");

    let db_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Could not connect to the database");

    let write_db_pool = PgPoolOptions::new()
        .connect(&write_database_url)
        .await
        .expect("Could not connect to the write database");

    let redis_conn = match connect_redis().await {
        Ok(con) => con,
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to Redis",
            ));
        }
    };

    let elasticsearch_url =
        std::env::var("ELASTICSEARCH_URL").expect("ELASTICSEARCH_URL must be set");
    let elasticsearch_username =
        std::env::var("ELASTICSEARCH_USERNAME").expect("ELASTICSEARCH_USERNAME must be set");
    let elasticsearch_password =
        std::env::var("ELASTICSEARCH_PASSWORD").expect("ELASTICSEARCH_PASSWORD must be set");
    let mut es_config = HashMap::new();
    es_config.insert("url".to_string(), elasticsearch_url);
    es_config.insert("username".to_string(), elasticsearch_username);
    es_config.insert("password".to_string(), elasticsearch_password);

    let db_pools = Arc::new([db_pool.clone(), write_db_pool.clone()]);

    HttpServer::new(move || {
        let cors = Cors::default()
            // Maybe we need to add some origin for security reason.
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(DefaultHeaders::new().add(("X-GIT-REVISION", env!("GIT_HASH", "N/A"))))
            .app_data(web::Data::new(db_pools.clone()))
            .app_data(web::Data::new(redis_conn.clone()))
            .app_data(web::Data::new(es_config.clone()))
            .configure(token::config)
            .configure(default_handler::configure)
            .configure(collection_handler::configure)
            .configure(token_handler::configure)
            .configure(portfolio_handler::configure)
            .service(web::scope("/v1").service(default_handler::health_check_v1))
            .service(api_doc::configure_docs())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn connect_redis() -> Result<Arc<Mutex<MultiplexedConnection>>, Box<dyn Error>> {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL not set");
    let redis_username = std::env::var("REDIS_USERNAME").expect("REDIS_USERNAME not set");
    let redis_password = std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD not set");

    let client = Client::open(format!(
        "redis://{}:{}@{}",
        redis_username, redis_password, redis_url
    ))?;
    let connection = client.get_multiplexed_tokio_connection().await?;
    Ok(Arc::new(Mutex::new(connection)))
}

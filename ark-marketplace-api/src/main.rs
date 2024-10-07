mod api_doc;

use actix_cors::Cors;
use actix_web::middleware::DefaultHeaders;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use ark_marketplace_api::routes::token;
use ark_marketplace_api::utils::app_config::AppConfig;
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_secretsmanager::config::{Credentials};
use redis::{aio::MultiplexedConnection, Client};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;
use aws_sdk_secretsmanager::Client as AwsClient;

use ark_marketplace_api::handlers::{
    collection_handler, default_handler, portfolio_handler, token_handler,
};

use clap::Parser;

// Default alocator change
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
/// Microservice that parse indexed block and push transactions to databse
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    config_path: String,
}

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

#[derive(Deserialize)]
struct RedisCredentials {
    url: String,
    username: String,
    password: String,
    port: u16,
}

#[derive(Deserialize)]
struct ElasticCredentials {
    url: String,
    username: String,
    password: String,
}

async fn get_database_url(client: AwsClient, secret: String) -> Result<String> {
    match std::env::var("DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(_) => {
            let secret_value = client
                .get_secret_value()
                .secret_id(secret)
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

async fn get_write_database_url(client: AwsClient, secret: String) -> Result<String> {
    match std::env::var("WRITE_DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(_) => {
            let secret_value = client
                .get_secret_value()
                .secret_id(secret)
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
    let args = Args::parse();
    init_logging();
    let config = AppConfig::load_from_file(&args.config_path);
    match config {
        Ok(config) => {
            println!("starting api marketplace....");
            dotenv::dotenv().ok();

            let region_provider = RegionProviderChain::first_try(Region::new(config.aws_default_region.clone()));
            let credentials = Credentials::new(
                &config.aws_access_key_id,
                &config.aws_secret_access_key,
                None,
                None,
                "api-marketplace"
            );
            let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;
            let client = aws_sdk_secretsmanager::Client::new(&aws_config);


            let database_url = get_database_url(client.clone(), config.aws_secret_read_db)
            .await
            .expect("Could not get the database URL");
    
            let write_database_url = get_write_database_url(client.clone(), config.aws_secret_write_db)
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
        
            let redis_conn = match connect_redis(client.clone(), config.aws_secret_redis_db).await {
                Ok(con) => con,
                Err(e) => {
                    tracing::error!("Failed to connect to Redis: {}", e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to connect to Redis",
                    ));
                }
            };
        
            let es_config = match get_elastic_config(client.clone(), config.aws_secret_eleasticsearch_db).await {
                Ok(es_config) => es_config,
                Err(e) => {
                    tracing::error!("Failed to connect to Redis: {}", e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to get Elastic configuration",
                    ));
                }
            };
        
        
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
                    .service(api_doc::configure())
            })
            .bind("0.0.0.0:8080")?
            .run()
            .await
        },
        Err(error) => panic!("{:#?}", error),
    }
}


async fn get_elastic_config(client: AwsClient, secret_name: String) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut es_config: HashMap<String, String> = HashMap::new();
    let secret_value = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    let result = secret_value.secret_string.unwrap();
    let creds: ElasticCredentials = serde_json::from_str(&result)?;
    let elasticsearch_url =
                std::env::var("ELASTICSEARCH_URL").unwrap_or(creds.url);
    let elasticsearch_username =
                std::env::var("ELASTICSEARCH_USERNAME").unwrap_or(creds.username);
    let elasticsearch_password =
                std::env::var("ELASTICSEARCH_PASSWORD").unwrap_or(creds.password);
    es_config.insert("url".to_string(), elasticsearch_url);
    es_config.insert("username".to_string(), elasticsearch_username);
    es_config.insert("password".to_string(), elasticsearch_password);
    Ok(es_config)
}

async fn connect_redis(client: AwsClient, secret_name: String) -> Result<Arc<Mutex<MultiplexedConnection>>, Box<dyn Error>> {
    let secret_value = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    let result = secret_value.secret_string.unwrap();
    let creds: RedisCredentials = serde_json::from_str(&result)?;
    let redis_url = std::env::var("REDIS_URL").unwrap_or(creds.url);
    let redis_username = std::env::var("REDIS_USERNAME").unwrap_or(creds.username);
    let redis_password = std::env::var("REDIS_PASSWORD").unwrap_or(creds.password);

    println!("redis port: {}", creds.port);

    let client = Client::open(format!(
        "redis://{}:{}@{}",
        redis_username, redis_password, redis_url
    ))?;
    let connection = client.get_multiplexed_tokio_connection().await?;
    Ok(Arc::new(Mutex::new(connection)))
}

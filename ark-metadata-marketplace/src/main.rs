extern crate openssl;
extern crate openssl_probe;

mod aws_s3_file_manager;
mod elasticsearch_manager;
mod metadata_storage;

use crate::aws_s3_file_manager::AWSFileManager;
use crate::elasticsearch_manager::EsManager;
use anyhow::Result;
use ark_metadata_marketplace::utils::app_config::OutputConfig;
use arkproject::{
    metadata::{
        metadata_manager::{MetadataError, MetadataManager},
        storage::Storage,
    },
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_secretsmanager::Client as AwsClient;
use clap::Parser;
use elasticsearch_manager::EsConfig;
use metadata_storage::MetadataSqlStorage;
use serde::Deserialize;
use std::{error::Error, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error, info, span, trace, warn, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_path: String,
}

#[derive(Deserialize)]
struct DatabaseCredentials {
    username: String,
    password: String,
    dbname: String,
    port: u16,
    host: String,
}

async fn get_database_url(client: AwsClient, secret: String) -> Result<String> {
    match std::env::var("DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(_) => {
            let secret_value = client.get_secret_value().secret_id(secret).send().await?;
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

async fn get_elastic_config(
    client: AwsClient,
    secret_name: String,
) -> Result<EsConfig, Box<dyn Error>> {
    let secret_value = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    let result = secret_value.secret_string.unwrap();
    let creds: EsConfig = serde_json::from_str(&result)?;
    let elasticsearch_url = std::env::var("ELASTICSEARCH_URL").unwrap_or(creds.url);
    let elasticsearch_username = std::env::var("ELASTICSEARCH_USERNAME").unwrap_or(creds.username);
    let elasticsearch_password = std::env::var("ELASTICSEARCH_PASSWORD").unwrap_or(creds.password);
    Ok(EsConfig {
        url: elasticsearch_url,
        username: elasticsearch_username,
        password: elasticsearch_password,
    })
}
#[derive(Debug, Deserialize, Clone)]
pub struct BucketConfig {
    pub name: String,
}

async fn get_bucket_config(
    client: AwsClient,
    secret_name: String,
) -> Result<BucketConfig, Box<dyn Error>> {
    let secret_value = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    let result = secret_value.secret_string.unwrap();
    let creds: BucketConfig = serde_json::from_str(&result)?;
    Ok(creds)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    init_tracing();
    let config = OutputConfig::load_from_file(&args.config_path);
    match config {
        Ok(config) => {
            println!("starting ark metadata marketplace");
            dotenv::dotenv().ok();

            let region_provider =
                RegionProviderChain::first_try(Region::new(config.aws_default_region.clone()));
            let credentials = Credentials::new(
                &config.aws_access_key_id,
                &config.aws_secret_access_key,
                None,
                None,
                "api-marketplace",
            );
            let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                .region(region_provider)
                .credentials_provider(credentials)
                .load()
                .await;
            let client = aws_sdk_secretsmanager::Client::new(&aws_config);
            let database_uri = get_database_url(client.clone(), config.aws_secret_read_db)
                .await
                .expect("Could not get the database URL");
            let storage = MetadataSqlStorage::new_pg(database_uri.as_str())
                .await
                .expect("Could not get the Storage");
            let starknet_client = StarknetClientHttp::new(&config.rcp_provider)
                .expect("Could not get the RPC provider");
            let bucket = get_bucket_config(client.clone(), config.aws_secret_bucket_name)
                .await
                .expect("Could not get the bucket Name");
            let file_manager = AWSFileManager::new(bucket.name);
            let es_config =
                match get_elastic_config(client.clone(), config.aws_secret_eleasticsearch_db).await
                {
                    Ok(es_config) => es_config,
                    Err(e) => {
                        tracing::error!("Failed to connect to AWS SECRET MANAGER: {}", e);
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to get Elastic configuration",
                        ));
                    }
                };
            let elasticsearch_manager =
                EsManager::new(es_config.url, es_config.username, es_config.password);

            trace!(
                "Initialized AWSFileManager, StarknetClientHttp, MetadataStorage and ElasticsearchManager"
            );
            let mut metadata_manager = MetadataManager::new(
                &storage,
                &starknet_client,
                &file_manager,
                Some(&elasticsearch_manager),
            );

            debug!("Starting main loop to check and refresh token metadata");
            if let Some((contract_address, chain_id)) = &config.filter {
                if config.refresh_contract_metadata {
                    info!(
                        "♻️ Forcing Refresh for NFT collection: {{ contract_address: \"{}\", chain_id: \"{}\" }}",
                        contract_address, chain_id
                    );

                    storage
                        .update_all_token_metadata_status(
                            contract_address,
                            chain_id,
                            "COLLECTION_TO_REFRESH",
                        )
                        .await
                        .expect("Error on update all token metadata Status");
                }
            }

            let mut total_tokens: u64 = 0;
            loop {
                match storage
                    .find_tokens_without_metadata(
                        config.filter.clone(),
                        config.refresh_contract_metadata,
                    )
                    .await
                {
                    Ok(tokens) => {
                        if tokens.is_empty() {
                            if config.refresh_contract_metadata {
                                info!("All collections metadata refreshed successfully");

                                if let Some((contract_address, chain_id)) = &config.filter {
                                    storage
                                        .set_contract_refreshing_status(
                                            contract_address,
                                            chain_id,
                                            false,
                                        )
                                        .await
                                        .expect("Error when refreshing contract status");
                                }

                                return Ok(());
                            }

                            info!("No tokens found that require metadata refresh");
                            sleep(
                                humantime::parse_duration(&config.loop_delay_duration)
                                    .unwrap_or(Duration::from_secs(60)),
                            )
                            .await;
                        } else {
                            for token in tokens {
                                total_tokens += 1;

                                info!(
                                    "🔄 Refreshing Token Metadata [{}]: Token: {:?}",
                                    total_tokens, token
                                );

                                match metadata_manager
                                    .refresh_token_metadata(
                                        &token.contract_address,
                                        &token.token_id,
                                        &token.chain_id,
                                        token.save_images,
                                        config.ipfs_gateway_uri.as_str(),
                                        humantime::parse_duration(&config.ipfs_timeout_duration)
                                            .unwrap_or(Duration::from_secs(60)),
                                        "https://arkproject.dev",
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        info!(
                                            "✅ Metadata for Token ID: {} refreshed successfully",
                                            token.token_id
                                        );
                                    }
                                    Err(metadata_error) => {
                                        match metadata_error {
                                            MetadataError::ParsingError(error) => {
                                                warn!("❌ Parsing error: {:?}", error);
                                            }
                                            e => {
                                                error!("❌ Error: {:?}", e);
                                            }
                                        }

                                        let _ = storage
                                            .update_token_metadata_status(
                                                &token.contract_address,
                                                &token.token_id,
                                                &token.chain_id,
                                                "ERROR",
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        sleep(
                            humantime::parse_duration(&config.loop_delay_duration)
                                .unwrap_or(Duration::from_secs(60)),
                        )
                        .await;
                    }
                }
            }
        }
        Err(error) => panic!("{:#?}", error),
    }
}

fn init_tracing() {
    // Initialize the LogTracer to convert `log` records to `tracing` events
    tracing_log::LogTracer::init().expect("Setting log tracer failed.");

    // Create the layers
    let env_filter = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer();

    // Combine layers and set as global default
    let subscriber = Registry::default().with(env_filter).with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed.");

    let main_span = span!(Level::TRACE, "main");
    let _main_guard = main_span.enter();
}

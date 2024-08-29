extern crate openssl;
extern crate openssl_probe;

mod aws_s3_file_manager;
mod elasticsearch_manager;
mod metadata_storage;

use crate::aws_s3_file_manager::AWSFileManager;
use crate::elasticsearch_manager::EsManager;
use anyhow::Result;
use arkproject::{
    metadata::{
        metadata_manager::{MetadataError, MetadataManager},
        storage::Storage,
    },
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use aws_config::BehaviorVersion;
use dotenv::dotenv;
use metadata_storage::MetadataSqlStorage;
use serde::Deserialize;
use std::{env, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error, info, span, trace, warn, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

struct Config {
    bucket_name: String,
    rpc_url: String,
    ipfs_timeout_duration: Duration,
    loop_delay_duration: Duration,
    ipfs_gateway_uri: String,
    filter: Option<(String, String)>,
    refresh_contract_metadata: bool,
    elasticsearch_url: String,
    elasticsearch_username: String,
    elasticsearch_password: String,
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

fn get_env_variables() -> Config {
    dotenv().ok();

    let bucket_name =
        env::var("AWS_NFT_IMAGE_BUCKET_NAME").expect("AWS_NFT_IMAGE_BUCKET_NAME must be set");
    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
    let ipfs_timeout_duration = Duration::from_secs(
        env::var("METADATA_IPFS_TIMEOUT_IN_SEC")
            .expect("METADATA_IPFS_TIMEOUT_IN_SEC must be set")
            .parse::<u64>()
            .expect("Invalid METADATA_IPFS_TIMEOUT_IN_SEC"),
    );
    let loop_delay_duration = Duration::from_secs(
        env::var("METADATA_LOOP_DELAY_IN_SEC")
            .expect("METADATA_LOOP_DELAY_IN_SEC must be set")
            .parse::<u64>()
            .expect("Invalid METADATA_LOOP_DELAY_IN_SEC"),
    );
    let ipfs_gateway_uri = env::var("IPFS_GATEWAY_URI").expect("IPFS_GATEWAY_URI must be set");

    let refresh_contract_metadata = env::var("REFRESH_CONTRACT_METADATA")
        .map(|value| value == "true")
        .unwrap_or(false);

    let filter = match env::var("METADATA_CONTRACT_FILTER") {
        Ok(contract_address) => {
            let chain_id = env::var("CHAIN_ID_FILTER").expect("CHAIN_ID_FILTER must be set");
            Some((contract_address, chain_id))
        }
        Err(_) => None,
    };

    // elasticsearch
    let elasticsearch_url = env::var("ELASTICSEARCH_URL").expect("ELASTICSEARCH_URL must be set");
    let elasticsearch_username =
        env::var("ELASTICSEARCH_USERNAME").expect("ELASTICSEARCH_USERNAME must be set");
    let elasticsearch_password =
        env::var("ELASTICSEARCH_PASSWORD").expect("ELASTICSEARCH_PASSWORD must be set");

    Config {
        bucket_name,
        rpc_url,
        ipfs_timeout_duration,
        loop_delay_duration,
        ipfs_gateway_uri,
        filter,
        refresh_contract_metadata,
        elasticsearch_url,
        elasticsearch_username,
        elasticsearch_password,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let config = get_env_variables();
    let database_uri = get_database_url().await?;

    let storage = MetadataSqlStorage::new_pg(database_uri.as_str()).await?;
    let starknet_client = StarknetClientHttp::new(&config.rpc_url)?;
    let file_manager = AWSFileManager::new(config.bucket_name);
    let elasticsearch_manager = EsManager::new(
        config.elasticsearch_url,
        config.elasticsearch_username,
        config.elasticsearch_password,
    );

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
                .await?;
        }
    }

    let mut total_tokens: u64 = 0;
    loop {
        match storage
            .find_tokens_without_metadata(config.filter.clone(), config.refresh_contract_metadata)
            .await
        {
            Ok(tokens) => {
                if tokens.is_empty() {
                    if config.refresh_contract_metadata {
                        info!("All collections metadata refreshed successfully");

                        if let Some((contract_address, chain_id)) = &config.filter {
                            storage
                                .set_contract_refreshing_status(contract_address, chain_id, false)
                                .await?;
                        }

                        return Ok(());
                    }

                    info!("No tokens found that require metadata refresh");
                    sleep(config.loop_delay_duration).await;
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
                                config.ipfs_timeout_duration,
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
                sleep(config.loop_delay_duration).await;
            }
        }
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

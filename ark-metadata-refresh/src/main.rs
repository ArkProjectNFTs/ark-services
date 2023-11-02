mod aws_s3_file_manager;

use crate::aws_s3_file_manager::AWSFileManager;
use anyhow::Result;
use ark_dynamodb::metadata_storage::MetadataStorage;
use arkproject::{
    metadata::{
        metadata_manager::{ImageCacheOption, MetadataError, MetadataManager},
        storage::Storage,
    },
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use dotenv::dotenv;
use starknet::core::types::FieldElement;
use std::{env, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error, info, span, trace, warn, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

struct Config {
    table_name: String,
    bucket_name: String,
    rpc_url: String,
    ipfs_timeout_duration: Duration,
    loop_delay_duration: Duration,
    ipfs_gateway_uri: String,
    contract_address_filter: Option<FieldElement>,
}

fn get_env_variables() -> Config {
    dotenv().ok();

    let table_name = env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set");
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

    let contract_address_filter = env::var("METADATA_CONTRACT_FILTER")
        .ok()
        .map(|value| FieldElement::from_hex_be(&value).expect("Invalid METADATA_CONTRACT_FILTER"));

    Config {
        table_name,
        bucket_name,
        rpc_url,
        ipfs_timeout_duration,
        loop_delay_duration,
        ipfs_gateway_uri,
        contract_address_filter,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let config = get_env_variables();
    let metadata_storage = MetadataStorage::new(config.table_name.clone()).await;
    let starknet_client = StarknetClientHttp::new(&config.rpc_url)?;
    let file_manager = AWSFileManager::new(config.bucket_name);

    trace!("Initialized AWSFileManager, StarknetClientHttp, and MetadataStorage");

    let mut metadata_manager =
        MetadataManager::new(&metadata_storage, &starknet_client, &file_manager);

    debug!("Starting main loop to check and refresh token metadata");

    loop {
        match metadata_storage
            .find_token_ids_without_metadata(config.contract_address_filter)
            .await
        {
            Ok(tokens) => {
                if tokens.is_empty() {
                    info!("No tokens found that require metadata refresh");
                    sleep(config.loop_delay_duration).await;
                    continue;
                } else {
                    for token in tokens {
                        let (contract_address, token_id) = token;

                        info!(
                            "🔄 Refreshing metadata. Contract address: 0x{:064x} - Token ID: {}",
                            contract_address,
                            token_id.to_decimal(false)
                        );

                        match metadata_manager
                            .refresh_token_metadata(
                                contract_address,
                                token_id.clone(),
                                ImageCacheOption::Save,
                                config.ipfs_gateway_uri.as_str(),
                                config.ipfs_timeout_duration,
                            )
                            .await
                        {
                            Ok(_) => {
                                info!(
                                    "✅ Metadata for Token ID: {} refreshed successfully",
                                    token_id.to_decimal(false)
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

                                let _ = metadata_storage
                                    .update_token_metadata_status(
                                        contract_address,
                                        token_id.clone(),
                                        "ERROR",
                                    )
                                    .await;
                            }
                        }
                    }
                    continue;
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
                sleep(config.loop_delay_duration).await;
                continue;
            }
        };
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

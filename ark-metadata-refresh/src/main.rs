mod aws_s3_file_manager;

use anyhow::Result;
use ark_dynamodb::metadata_storage::MetadataStorage;
use arkproject::{
    metadata::{
        metadata_manager::{ImageCacheOption, MetadataManager},
        storage::Storage,
    },
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use core::panic;
use dotenv::dotenv;
use starknet::core::types::FieldElement;
use std::{env, time::Duration};
use tokio::time::sleep;
use tracing::{error, info, span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use crate::aws_s3_file_manager::AWSFileManager;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();
    let table_name: String =
        env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set");
    let bucket_name =
        env::var("AWS_NFT_IMAGE_BUCKET_NAME").expect("AWS_NFT_IMAGE_BUCKET_NAME must be set");
    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");

    let ipfs_timeout_duration = match env::var("METADATA_IPFS_TIMEOUT_IN_SEC") {
        Ok(value) => {
            let timeout = value
                .parse::<u64>()
                .expect("Invalid METADATA_IPFS_TIMEOUT_IN_SEC");
            Duration::from_secs(timeout)
        }
        Err(_) => {
            panic!("METADATA_IPFS_TIMEOUT_IN_SEC must be set");
        }
    };

    let loop_delay_duration = match env::var("METADATA_LOOP_DELAY_IN_SEC") {
        Ok(value) => {
            let timeout = value
                .parse::<u64>()
                .expect("Invalid METADATA_LOOP_DELAY_IN_SEC");
            Duration::from_secs(timeout)
        }
        Err(_) => {
            panic!("METADATA_LOOP_DELAY_IN_SEC must be set");
        }
    };

    let metadata_storage = MetadataStorage::new(table_name).await;
    let starknet_client = StarknetClientHttp::new(&rpc_url)?;
    let file_manager = AWSFileManager::new(bucket_name);

    let ipfs_gateway_uri = env::var("IPFS_GATEWAY_URI").expect("IPFS_GATEWAY_URI must be set");
    let mut metadata_manager =
        MetadataManager::new(&metadata_storage, &starknet_client, &file_manager);

    let contract_address_filter = match env::var("METADATA_CONTRACT_FILTER") {
        Ok(value) => {
            let contract_address_field_element = FieldElement::from_hex_be(value.as_str())
                .expect("Invalid METADATA_CONTRACT_FILTER");
            Some(contract_address_field_element)
        }
        Err(_) => None,
    };

    loop {
        match metadata_storage
            .find_token_ids_without_metadata(contract_address_filter)
            .await
        {
            Ok(tokens) => {
                if tokens.is_empty() {
                    info!("No tokens to refresh (without metadata)");
                    sleep(loop_delay_duration).await;
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
                                token_id,
                                ImageCacheOption::Save,
                                ipfs_gateway_uri.as_str(),
                                ipfs_timeout_duration,
                            )
                            .await
                        {
                            Ok(_) => info!("✅ Metadata refreshed successfully"),
                            Err(e) => error!("Error: {:?}", e),
                        }
                    }
                    continue;
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
                sleep(loop_delay_duration).await;
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

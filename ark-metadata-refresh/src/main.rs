mod aws_s3_file_manager;

use std::{env, time::Duration};

use anyhow::Result;
use ark_dynamodb::metadata_storage::MetadataStorage;
use arkproject::{
    metadata::{
        file_manager::LocalFileManager,
        metadata_manager::{CacheOption, MetadataManager},
        storage::Storage,
    },
    starknet::client::{StarknetClient, StarknetClientHttp},
};

use dotenv::dotenv;
use tokio::time::sleep;
use tracing::{error, info, span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();
    info!("Starting metadata refresh");

    let table_name: String =
        env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set");
    let metadata_storage = MetadataStorage::new(table_name).await;
    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
    let starknet_client = StarknetClientHttp::new(&rpc_url)?;

    // let file_manager = AWSFileManager::new("".to_string());
    let file_manager = LocalFileManager::default();

    let ipfs_gateway_uri = env::var("IPFS_GATEWAY_URI").expect("IPFS_GATEWAY_URI must be set");
    let mut metadata_manager =
        MetadataManager::new(&metadata_storage, &starknet_client, &file_manager);

    loop {
        match metadata_storage.find_token_ids_without_metadata().await {
            Ok(tokens) => {
                if tokens.len() == 0 {
                    info!("No tokens to refresh");
                    sleep(Duration::from_secs(10)).await;
                    continue;
                } else {
                    for (contract_address, token_id) in tokens {
                        match metadata_manager
                            .refresh_token_metadata(
                                contract_address,
                                token_id,
                                CacheOption::Cache,
                                ipfs_gateway_uri.as_str(),
                            )
                            .await
                        {
                            Ok(_) => info!("Success"),
                            Err(e) => error!("Error: {:?}", e),
                        }
                    }
                    continue;
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
                sleep(Duration::from_secs(10)).await;
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

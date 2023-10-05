mod aws_s3_file_manager;
mod metadata_storage;

use std::env;

use anyhow::Result;
use arkproject::{starknet::{CairoU256, client::{StarknetClient, StarknetClientHttp}}, metadata::metadata_manager::MetadataManager};
use aws_s3_file_manager::AWSFileManager;
use dotenv::dotenv;
use metadata_storage::MetadataStorage;
use starknet::core::types::FieldElement;
use tracing::{span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();

    let table_name: String = env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set");
    let metadata_storage = MetadataStorage::new("".to_string()).await;
    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
    let starknet_client = StarknetClientHttp::new(&rpc_url)?;
    let file_manager = AWSFileManager::new("".to_string());

    let metadata_manager = MetadataManager::new(&metadata_storage, &starknet_client, &file_manager);
    
    Ok(())
}

pub fn init_tracing() {
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

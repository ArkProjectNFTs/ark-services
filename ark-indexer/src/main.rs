mod aws_s3_file_manager;
mod dynamo_storage;
mod pontos_observer;

use crate::dynamo_storage::DynamoStorage;
use anyhow::Result;
use arkproject::{
    pontos::{Pontos, PontosConfig},
    starknet::client::{StarknetClient, StarknetClientHttp},
};

use dotenv::dotenv;
use pontos_observer::PontosObserver;
use regex::Regex;
use starknet::core::types::{BlockId, BlockTag};
use std::{env, sync::Arc};
use tracing::{debug, info, trace};
use tracing::{span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    init_tracing();

    let is_head_of_chain = env::var("HEAD_OF_CHAIN").is_ok();

    let (from_block, to_block) = if is_head_of_chain {
        (BlockId::Number(0), BlockId::Number(0))
    } else {
        let from = BlockId::Number(
            env::var("FROM_BLOCK")
                .expect("FROM_BLOCK must be set")
                .parse()
                .expect("Can't parse FROM_BLOCK, expecting u64"),
        );

        let to: BlockId = match env::var("TO_BLOCK") {
            Ok(to) => BlockId::Number(to.parse().expect("Can't parse TO_BLOCK, expecting u64")),
            Err(_) => BlockId::Tag(BlockTag::Latest),
        };

        (from, to)
    };

    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
    let table_name = env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set");
    let force_mode = env::var("FORCE_MODE").is_ok();
    let indexer_version = env::var("INDEXER_VERSION").expect("INDEXER_VERSION must be set");
    let indexer_identifier = get_task_id();

    info!(
        "🏁 Starting Indexer. Version={}, Identifier={}",
        indexer_version, indexer_identifier
    );

    debug!(
        "from_block={:?}, to_block={:?}, head_of_the_chain={}, rpc_url={}, table_name={}, force_mode={}, indexer_version={}, indexer_identifier={}",
       from_block, to_block, is_head_of_chain, rpc_url, table_name, force_mode, indexer_version, indexer_identifier
    );

    let dynamo_storage = Arc::new(DynamoStorage::new(table_name.clone()).await);
    let starknet_client = Arc::new(StarknetClientHttp::new(rpc_url.as_str())?);

    let pontos_observer = Arc::new(PontosObserver::new(
        Arc::clone(&dynamo_storage),
        indexer_version.clone(),
        indexer_identifier.clone(),
    ));

    let pontos_task = Pontos::new(
        Arc::clone(&starknet_client),
        dynamo_storage,
        Arc::clone(&pontos_observer),
        PontosConfig {
            indexer_version,
            indexer_identifier,
        },
    );

    if is_head_of_chain {
        trace!("Syncing Pontos at head of the chain");
        pontos_task.index_pending().await?;
    } else {
        trace!(
            "Syncing Pontos for block range: {:?} - {:?}",
            from_block,
            to_block
        );
        pontos_task
            .index_block_range(from_block, to_block, force_mode)
            .await?;
    }

    Ok(())
}

fn get_task_id() -> String {
    match env::var("ECS_CONTAINER_METADATA_URI") {
        Ok(container_metadata_uri) => {
            let pattern = Regex::new(r"/v3/([a-f0-9]{32})-").unwrap();
            let task_id = pattern
                .captures(container_metadata_uri.as_str())
                .and_then(|cap| cap.get(1).map(|m| m.as_str()))
                .expect("Can't parse task id from ECS_CONTAINER_METADATA_URI");

            task_id.to_string()
        }
        Err(_) => {
            if env::var("HEAD_OF_CHAIN").is_ok() {
                String::from("LATEST")
            } else {
                String::from("LOCALHOST")
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

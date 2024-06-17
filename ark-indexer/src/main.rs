mod pontos_observer;
use anyhow::Result;
use ark_dynamodb::storage::DynamoStorage;
use arkproject::{
    pontos::{Pontos, PontosConfig},
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use dotenv::dotenv;
use pontos_observer::PontosObserver;
use regex::Regex;
use starknet::core::types::{BlockId, FieldElement};
use std::{env, sync::Arc};
use tracing::{debug, info, trace};
use tracing::{span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    init_tracing();

    let is_head_of_chain = match std::env::var("HEAD_OF_CHAIN") {
        Ok(val) => val == "true",
        Err(_) => false,
    };

    let (from_block, to_block) = if is_head_of_chain {
        (None, None)
    } else {
        let from_value = env::var("FROM_BLOCK")
            .ok()
            .and_then(|val| val.parse::<u64>().map(BlockId::Number).ok());

        let to_value = env::var("TO_BLOCK")
            .ok()
            .and_then(|val| val.parse::<u64>().map(BlockId::Number).ok());

        (from_value, to_value)
    };

    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
    let table_name = env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set");
    let force_mode = env::var("FORCE_MODE").is_ok();
    let indexer_version = env::var("INDEXER_VERSION").ok();
    let chain_id = env::var("CHAIN_ID").expect("CHAIN_ID must be set");
    let indexer_identifier = get_task_id(is_head_of_chain);
    let block_indexer_function_name = match env::var("BLOCK_INDEXER_FUNCTION_NAME") {
        Ok(val) => Some(val),
        Err(_) => None,
    };
    let contract_address = env::var("CONTRACT_ADDRESS")
        .ok()
        .map(|value| FieldElement::from_hex_be(&value).expect("Invalid CONTRACT_ADDRESS"));

    info!(
        "🏁 Starting Indexer. Version={:?}, Identifier={}",
        indexer_version, indexer_identifier
    );

    debug!(
        "from_block={:?}, to_block={:?}, head_of_the_chain={}, rpc_url={}, table_name={}, force_mode={}, indexer_version={:?}, indexer_identifier={}, block_indexer_function_name={:?}, contract_address={:?}",
       from_block, to_block, is_head_of_chain, rpc_url, table_name, force_mode, indexer_version, indexer_identifier, block_indexer_function_name, contract_address
    );

    let dynamo_storage = Arc::new(DynamoStorage::new(table_name.clone()).await);
    let starknet_client = Arc::new(StarknetClientHttp::new(rpc_url.as_str())?);

    let pontos_observer = Arc::new(PontosObserver::new(
        Arc::clone(&dynamo_storage),
        indexer_version.clone(),
        indexer_identifier.clone(),
        block_indexer_function_name.clone(),
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
    // If syncing at the head of the chain
    if is_head_of_chain {
        trace!("Syncing Pontos at head of the chain");
        pontos_task.index_pending().await?;
        return Ok(());
    }

    // Proceed only if not at the head of the chain
    trace!(
        "Syncing Pontos for block range: {:?} - {:?}",
        from_block,
        to_block
    );

    // If a contract address is specified, index contract events
    if let Some(contract_address) = contract_address {
        pontos_task
            .index_contract_events(from_block, to_block, contract_address, chain_id.as_str())
            .await?;
        return Ok(());
    }

    // If both from_block and to_block are specified, index the block range
    if let (Some(from_block), Some(to_block)) = (from_block, to_block) {
        pontos_task
            .index_block_range(from_block, to_block, force_mode, chain_id.as_str())
            .await?;
        return Ok(());
    }

    // Optionally, handle the case where either from_block or to_block is None, if needed
    // This might include logging a warning or error if these values are expected to be present

    Ok(())
}

fn get_task_id(is_head_of_chain: bool) -> String {
    match env::var("ECS_CONTAINER_METADATA_URI") {
        Ok(container_metadata_uri) => {
            debug!("ECS_CONTAINER_METADATA_URI={}", container_metadata_uri);
            let pattern = Regex::new(r"/v3/([a-f0-9]{32})-").unwrap();
            let task_id = pattern
                .captures(container_metadata_uri.as_str())
                .and_then(|cap| cap.get(1).map(|m| m.as_str()))
                .expect("Can't parse task id from ECS_CONTAINER_METADATA_URI");

            task_id.to_string()
        }
        Err(_) => {
            if is_head_of_chain {
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

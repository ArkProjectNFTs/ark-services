mod pontos_observer;

use crate::pontos_observer::PontosObserver;
use ark_dynamodb::storage::DynamoStorage;
use arkproject::{
    pontos::{Pontos, PontosConfig},
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use dotenv::dotenv;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use starknet::core::types::BlockId;
use std::{env, sync::Arc};
use tracing::{info, trace};

#[derive(Deserialize, Serialize, Debug)]
struct BlockRange {
    from_block: u64,
    to_block: u64,
}

struct Config {
    rpc_url: String,
    table_name: String,
    force_mode: bool,
    indexer_version: String,
    indexer_identifier: String,
}

async fn get_config() -> Result<Config, Error> {
    Ok(Config {
        rpc_url: env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set"),
        table_name: env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set"),
        force_mode: env::var("FORCE_MODE").is_ok(),
        indexer_version: env::var("INDEXER_VERSION").expect("INDEXER_VERSION must be set"),
        indexer_identifier: "lambda-block-indexer".to_string(),
    })
}

#[derive(Serialize)]
struct Response {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<BlockRange>) -> Result<Response, Error> {
    trace!("Received request: {:?}", event);

    let block_range = event.payload;
    let from_block = BlockId::Number(block_range.from_block);
    let to_block = BlockId::Number(block_range.to_block);

    info!("🔄 Refresh block range: {:?} - {:?}", from_block, to_block);

    let config = get_config().await?;
    let dynamo_storage = Arc::new(DynamoStorage::new(config.table_name.clone()).await);
    let starknet_client = Arc::new(StarknetClientHttp::new(&config.rpc_url)?);

    let pontos_observer = Arc::new(PontosObserver::new(
        Arc::clone(&dynamo_storage),
        config.indexer_version.clone(),
        config.indexer_identifier.clone(),
    ));

    let pontos_task = Pontos::new(
        starknet_client,
        dynamo_storage,
        pontos_observer,
        PontosConfig {
            indexer_version: config.indexer_version,
            indexer_identifier: config.indexer_identifier,
        },
    );

    match pontos_task
        .index_block_range(from_block, to_block, config.force_mode)
        .await
    {
        Ok(_) => {
            info!("✅ Indexing completed");
            Ok(Response {
                message: "Indexing completed".to_string(),
            })
        }
        Err(err) => {
            info!("Indexing failed: {:?}", err);
            Err(Error::from("Indexing failed"))
        }
    }
}

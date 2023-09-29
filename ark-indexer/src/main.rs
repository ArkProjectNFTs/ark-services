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
use starknet::core::types::BlockId;
use std::{env, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
    let dynamo_storage = Arc::new(DynamoStorage::new().await);
    let starknet_client = Arc::new(StarknetClientHttp::new(rpc_url.as_str())?);

    let indexer_version = 1;
    let indexer_identifier = String::from("main");

    let pontos_observer = Arc::new(PontosObserver::new(
        Arc::clone(&dynamo_storage),
        indexer_version,
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

    pontos_task
        .index_block_range(BlockId::Number(80000), BlockId::Number(90000), true)
        .await?;

    Ok(())
}

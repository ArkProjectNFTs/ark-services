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
use starknet::core::types::{BlockId, BlockTag};
use std::{env, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let start_block_id = env::var("START_BLOCK")
        .expect("START_BLOCK must be set")
        .parse()
        .unwrap();

    let from_block: BlockId = BlockId::Number(start_block_id);
    let to_block: BlockId = match env::var("END_BLOCK") {
        Ok(end_block) => {
            let end_block: u64 = end_block.parse().unwrap();
            BlockId::Number(end_block)
        }
        Err(_) => BlockId::Tag(BlockTag::Latest),
    };

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
        .index_block_range(
           from_block,
           to_block,
            true,
        )
        .await?;

    Ok(())
}

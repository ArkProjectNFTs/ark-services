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
use starknet::core::types::{BlockId, BlockTag};
use std::{env, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

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

    let dynamo_storage = Arc::new(DynamoStorage::new(table_name.clone()).await);
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

    if is_head_of_chain {
        log::trace!("Syncing Pontos at head of the chain");
        pontos_task.index_pending().await?;
    } else {
        log::trace!(
            "Syncing Pontos for block range: {:?} - {:?}",
            from_block,
            to_block
        );
        pontos_task
            .index_block_range(from_block, to_block, true)
            .await?;
    }

    Ok(())
}

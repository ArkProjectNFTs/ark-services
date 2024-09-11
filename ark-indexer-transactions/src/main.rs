pub mod helpers;
pub mod interfaces;
pub mod services;

use std::sync::Arc;

use helpers::app_config::AppConfig;
use services::contract::manager::ContractManager;
use services::storage::block::{get_latest_block_in_folder, get_latest_folder_path};
use services::storage::database::DatabaseStorage;

use starknet::core::types::Felt;
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Url,
};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// Default alocator change
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("starting the block indexer for transactions....");
    let config = AppConfig::load_from_file();
    match config {
        Ok(config) => {
            let storage = DatabaseStorage::new(&config.database_url).await?;
            let provider = JsonRpcClient::new(HttpTransport::new(
                Url::parse(&config.rcp_provider).unwrap(),
            ));

            let mut contract_manager = ContractManager::new(
                Arc::new(Mutex::new(storage)),
                Arc::new(Mutex::new(provider)),
            );
            let chain_id = Felt::from_hex(&config.chain_id).unwrap_or(Felt::ZERO); // starknet mainnet chain ID
            loop {
                let latest_folder = get_latest_folder_path(&config.base_path)?;
                let lastest_block_number = get_latest_block_in_folder(&latest_folder)?;

                contract_manager
                    .index_blocks(
                        0,
                        lastest_block_number,
                        &config.parsing_state_path,
                        chain_id,
                    )
                    .await?;

                // let parsing_state = load_parsing_state(parsing_state_path)?;
                // if parsing_state.is_block_indexed(lastest_block_number) {
                //     contract_manager.reindex_pending_block(parsing_state_path, chain_id).await?;
                // }
                sleep(Duration::from_secs(1)).await;
            }
        }
        Err(error) => panic!("{:#?}", error),
    }
}

pub mod helpers;
pub mod interfaces;
pub mod services;

use helpers::app_config::AppConfig;
use services::contract::manager::ContractManager;
use services::storage::block::{get_latest_block_in_folder, get_latest_folder_path};
use services::storage::database::DatabaseStorage;

use clap::Parser;
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Url,
};
use starknet_crypto::Felt;
use tokio::time::{sleep, Duration};

// Default alocator change
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
/// Microservice that parse indexed block and push transactions to databse
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    config_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("starting the block indexer for transactions....");
    let config = AppConfig::load_from_file(&args.config_path);
    match config {
        Ok(config) => {
            let storage = DatabaseStorage::new(&config.database_url).await?;
            let provider = JsonRpcClient::new(HttpTransport::new(
                Url::parse(&config.rcp_provider).unwrap(),
            ));

            let orderbooks: Vec<Felt> = config
                .orderbooks
                .iter()
                .filter_map(|e| Felt::from_hex(e).ok())
                .collect();
            let mut contract_manager = ContractManager::new(storage, provider, orderbooks);
            // let chain_id = Felt::from_hex(&config.chain_id).unwrap_or(Felt::ZERO); // starknet mainnet chain ID
            loop {
                let latest_folder = get_latest_folder_path(&config.base_path)?;
                let mut lastest_block_number = get_latest_block_in_folder(&latest_folder)?;
                if let Some(end_at) = config.end_at {
                    lastest_block_number = end_at
                }
                contract_manager
                    .index_blocks(
                        config.start_from,
                        lastest_block_number,
                        &config.base_path,
                        &config.parsing_state_path,
                        &config.chain_id,
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

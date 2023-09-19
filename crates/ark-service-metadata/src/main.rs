use anyhow::Result;
use ark_rs::starknet::client::{StarknetClient, StarknetClientHttp};
use ark_rs::{
    nft_metadata::{file_manager::LocalFileManager, metadata_manager::MetadataManager},
    nft_storage::DefaultStorage,
};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let rpc_url = std::env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");

    let storage_manager = DefaultStorage::new();
    let starknet_client =
        StarknetClientHttp::new(&rpc_url).expect("Failed to create Starknet client");
    let file_manager = LocalFileManager::new();
    let metadata_manager = MetadataManager::new(&storage_manager, &starknet_client, &file_manager);

    // metadata_manager.fetch_token_image(url, name, cache_image);

    Ok(())
}

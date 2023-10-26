mod aws_s3_file_manager;

use crate::aws_s3_file_manager::AWSFileManager;
use ark_dynamodb::{
    metadata_storage::MetadataStorage,
    providers::{ArkTokenProvider, DynamoDbTokenProvider},
};
use arkproject::{
    metadata::metadata_manager::{ImageCacheOption, MetadataManager},
    starknet::{
        client::{StarknetClient, StarknetClientHttp},
        CairoU256,
    },
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError,
};
use starknet::core::types::FieldElement;
use std::{env, time::Duration};
use tracing::{error, info, warn};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let ctx = LambdaCtx::from_event(&event).await?;

    let provider = DynamoDbTokenProvider::new(&ctx.table_name, None);

    let (address, token_id_hex) = get_params(&event)?;

    info!(
        "Checking for existing token. Address: {}, Token ID (Hex): {}",
        address, token_id_hex
    );

    let rsp = provider.get_token(&ctx.db, &address, &token_id_hex).await?;

    if let Some(data) = rsp.inner() {
        info!(
            "🔄 Refreshing metadata. Contract address: {} - Token ID: {}",
            data.contract_address, data.token_id
        );

        let bucket_name =
            env::var("AWS_NFT_IMAGE_BUCKET_NAME").expect("AWS_NFT_IMAGE_BUCKET_NAME must be set");
        let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
        let table_name: String =
            env::var("INDEXER_TABLE_NAME").expect("INDEXER_TABLE_NAME must be set");
        let ipfs_gateway_uri = env::var("IPFS_GATEWAY_URI").expect("IPFS_GATEWAY_URI must be set");

        info!("bucket_name: {}", bucket_name);
        info!("rpc_url: {}", rpc_url);
        info!("table_name: {}", table_name);
        info!("ipfs_gateway_uri: {}", ipfs_gateway_uri);

        let metadata_storage = MetadataStorage::new(table_name).await;
        let starknet_client = StarknetClientHttp::new(&rpc_url)?;
        let file_manager = AWSFileManager::new(bucket_name);
        let mut metadata_manager =
            MetadataManager::new(&metadata_storage, &starknet_client, &file_manager);
        let contract_address_field_element =
            FieldElement::from_hex_be(&data.contract_address).expect("Invalid contract address");
        let token_id = CairoU256::from_hex_be(&data.token_id_hex).expect("Invalid token ID");

        let ipfs_timeout_duration = match env::var("METADATA_IPFS_TIMEOUT_IN_SEC") {
            Ok(value) => {
                let timeout = value
                    .parse::<u64>()
                    .expect("Invalid METADATA_IPFS_TIMEOUT_IN_SEC");
                Duration::from_secs(timeout)
            }
            Err(_) => {
                panic!("METADATA_IPFS_TIMEOUT_IN_SEC must be set");
            }
        };

        info!("refresh_token_metadata call");

        match metadata_manager
            .refresh_token_metadata(
                contract_address_field_element,
                token_id,
                ImageCacheOption::Save,
                ipfs_gateway_uri.as_str(),
                ipfs_timeout_duration,
            )
            .await
        {
            Ok(_) => {
                info!("✅ Metadata refreshed successfully");
                return common::ok_body_rsp(&ArkApiResponse {
                    cursor: None,
                    result: {},
                });
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return common::not_found_rsp();
            }
        };
    };

    warn!(
        "Token not found. Address: {}, Token ID (Hex): {}",
        address, token_id_hex
    );
    return common::not_found_rsp();
}

fn get_params(event: &Request) -> Result<(String, String), LambdaHttpError> {
    let address = match common::require_hex_param(event, "contract_address", HttpParamSource::Path)
    {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    let token_id_hex =
        match common::require_hex_or_dec_param(event, "token_id", HttpParamSource::Path) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

    Ok((address, token_id_hex))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(|event: Request| async {
        function_handler(event).await
    }))
    .await
}

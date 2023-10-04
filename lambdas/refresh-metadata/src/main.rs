mod aws_s3_file_manager;
mod storage;

use arkproject::{
    metadata::metadata_manager::MetadataManager,
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use aws_s3_file_manager::AWSFileManager;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use storage::DynamoStorage;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // let who = event
    //     .query_string_parameters_ref()
    //     .and_then(|params| params.first("name"))
    //     .unwrap_or("world");

    // let message = format!("Hello {who}, this is an AWS Lambda HTTP request");

    let rpc_url = &"";
    let starknet_client = StarknetClientHttp::new(rpc_url)?;
    let storage = DynamoStorage::default();
    let aws_file_manager = AWSFileManager::default();

    let metadata_manager = MetadataManager::new(&storage, &starknet_client, &aws_file_manager);
    // metadata_manager.refresh_token_metadata(contract_address, token_id, cache, ipfs_gateway_uri)?;

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(Body::from("Hello world!"))
        .map_err(Box::new)?;

    Ok(resp)
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

    run(service_fn(function_handler)).await
}

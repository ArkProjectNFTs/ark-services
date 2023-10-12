//! A playground to interact with dynamodb locally using providers.
//!
use ark_dynamodb::{init_aws_dynamo_client, providers::*, Client as DynamoClient};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common as common;

/// A struct to bundle all init required by the lambda.
struct Ctx {
    client: DynamoClient,
    _token: DynamoDbTokenProvider,
    _contract: DynamoDbContractProvider,
    _event: DynamoDbEventProvider,
    block: DynamoDbBlockProvider,
}

async fn function_handler(ctx: &Ctx, _event: Request) -> Result<Response<Body>, Error> {
    match ctx.block.clean(&ctx.client, 1694176591, None).await {
        Ok(_) => (),
        Err(e) => println!("ERR____ {:?}", e),
    };

    common::ok_body_rsp(&1)
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

    let table_name = std::env::var("ARK_TABLE_NAME").expect("ARK_TABLE_NAME must be set");

    let ctx = Ctx {
        client: init_aws_dynamo_client().await,
        _token: DynamoDbTokenProvider::new(&table_name),
        _contract: DynamoDbContractProvider::new(&table_name),
        _event: DynamoDbEventProvider::new(&table_name),
        block: DynamoDbBlockProvider::new(&table_name),
    };

    run(service_fn(|event: Request| async {
        function_handler(&ctx, event).await
    }))
    .await
}

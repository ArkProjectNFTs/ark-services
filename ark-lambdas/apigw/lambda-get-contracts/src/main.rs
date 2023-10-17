//! A Lambda function to get all contracts that has been identified as collections.
//!
//! To work, this lambda expects the following path:
//!     `../collections`
//!
use ark_dynamodb::providers::{ArkContractProvider, DynamoDbContractProvider};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, ArkApiResponse, LambdaCtx};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let ctx = LambdaCtx::from_event(&event).await?;

    let provider = DynamoDbContractProvider::new(&ctx.table_name, ctx.max_items_limit);

    let rsp = provider.get_contracts(&ctx.db).await?;

    let items = rsp.inner();
    let cursor = ctx.paginator.store_cursor(&rsp.lek)?;

    common::ok_body_rsp(&ArkApiResponse {
        cursor,
        result: items,
    })
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

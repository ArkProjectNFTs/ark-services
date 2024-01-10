//! A Lambda function to get all contracts that has been identified as collections.
//!
//! To work, this lambda expects the following path:
//!     `../collections`
//!
use ark_dynamodb::providers::{ArkContractProvider, DynamoDbContractProvider};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, ArkApiResponse, LambdaCtx, LambdaHttpResponse};
use std::collections::HashMap;
use tracing::{error, info};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // 1. Init the context.
    let ctx = LambdaCtx::from_event(&event).await?;

    // 2. No params.

    // 3. Process the request.
    let r = process_event(&ctx).await;

    // 4. Send response.
    let req_params = HashMap::new();

    match r {
        Ok(lambda_rsp) => {
            info!("Result: {:?}", lambda_rsp);
            ctx.register_usage(req_params, Some(&lambda_rsp)).await?;
            Ok(lambda_rsp.inner)
        }
        Err(e) => {
            error!("Error: {:?}", e);
            ctx.register_usage(req_params, None).await?;
            Err(e)
        }
    }
}

async fn process_event(ctx: &LambdaCtx) -> Result<LambdaHttpResponse, Error> {
    let provider = DynamoDbContractProvider::new(&ctx.table_name, ctx.max_items_limit);

    let dynamo_rsp = provider.get_nft_contracts(&ctx.dynamodb).await?;
    let items = dynamo_rsp.inner();
    let last_evaluated_key = &dynamo_rsp.lek;

    info!("Last evaluated key: {:?}", last_evaluated_key);

    let cursor = ctx.paginator.store_cursor(last_evaluated_key)?;

    info!("Cursor: {:?}", cursor);

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor,
        result: items,
    })?;

    info!("Response: {:?}", rsp);

    Ok(LambdaHttpResponse {
        capacity: dynamo_rsp.capacity,
        inner: rsp,
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

//! A Lambda function to get all the events for a contract.
//!
//! To work, this lambda expects the following path:
//!     `../events/{owner_address}`
//!
//! where:
//!   * owner_address: Contract address of the collection, in hexadecimal.
//!
//! Examples:
//! `https://.../owner/events/0x1234`
//!
use ark_dynamodb::pagination::Lek;
use ark_dynamodb::providers::{ArkEventProvider, DynamoDbEventProvider};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError, LambdaHttpResponse,
};
use std::collections::HashMap;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // 1. Init the context.
    let ctx = LambdaCtx::from_event(&event).await?;

    // 2. Get params.
    let owner_address = get_params(&event)?;

    // 3. Process the request.
    let r = process_event(&ctx, &owner_address).await;

    // 4. Send the response.
    let mut req_params = HashMap::new();
    req_params.insert("owner_address".to_string(), owner_address.clone());

    match r {
        Ok(lambda_rsp) => {
            ctx.register_usage(req_params, Some(&lambda_rsp)).await?;
            Ok(lambda_rsp.inner)
        }
        Err(e) => {
            ctx.register_usage(req_params, None).await?;
            Err(e)
        }
    }
}

async fn process_event(ctx: &LambdaCtx, owner_address: &str) -> Result<LambdaHttpResponse, Error> {
    let provider = DynamoDbEventProvider::new(&ctx.table_name, ctx.max_items_limit);

    let mut leks: HashMap<String, Option<Lek>> = HashMap::new();

    let dynamo_rsp_from = provider
        .get_owner_from_events(&ctx.dynamodb, owner_address, "from")
        .await?;

    leks.insert("from".to_string(), dynamo_rsp_from.lek.clone());

    let dynamo_rsp_to = provider
        .get_owner_to_events(&ctx.dynamodb, owner_address, "to")
        .await?;

    leks.insert("to".to_string(), dynamo_rsp_to.lek.clone());

    let cursor = ctx.paginator.store_cursor_multiple(&leks)?;

    // Combine items from both responses
    let mut items = dynamo_rsp_from.inner().clone();
    items.extend(dynamo_rsp_to.inner().clone());

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor,
        total_count: None,
        result: items,
    })?;

    Ok(LambdaHttpResponse {
        capacity: dynamo_rsp_to.capacity,
        inner: rsp,
    })
}

fn get_params(event: &Request) -> Result<String, LambdaHttpError> {
    common::require_hex_param(event, "owner_address", HttpParamSource::Path)
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

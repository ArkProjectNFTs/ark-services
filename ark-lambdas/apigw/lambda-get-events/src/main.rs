use ark_dynamodb::providers::{ArkEventProvider, DynamoDbEventProvider};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, ArkApiResponse, LambdaCtx, LambdaHttpResponse};
use tracing::{error, info};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let ctx = LambdaCtx::from_event(&event).await?;
    let r = process_event(&ctx).await;

    match r {
        Ok(lambda_rsp) => Ok(lambda_rsp.inner),
        Err(e) => {
            error!("Error: {:?}", e);
            Err(e)
        }
    }
}

async fn process_event(ctx: &LambdaCtx) -> Result<LambdaHttpResponse, Error> {
    info!(
        "Processing event... table_name={}, max_items_limit={:?}, exclusive_start_key={:?}",
        ctx.table_name, ctx.max_items_limit, ctx.dynamodb.exclusive_start_key
    );

    let provider = DynamoDbEventProvider::new(&ctx.table_name, ctx.max_items_limit);
    let dynamo_rsp = provider.get_events(&ctx.dynamodb).await?;

    info!("DynamoDB response: {:?}", dynamo_rsp);

    let items = dynamo_rsp.inner();
    let last_evaluated_key = &dynamo_rsp.lek;

    info!("Last evaluated key: {:?}", last_evaluated_key);

    let cursor = ctx.paginator.store_cursor(&dynamo_rsp.lek)?;

    info!("Cursor: {:?}", cursor);

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor,
        total_count: None,
        result: items,
    })?;

    info!("Response: {:?}", rsp);

    let capacity = dynamo_rsp.consumed_capacity_units.unwrap_or(0.0);

    Ok(LambdaHttpResponse {
        capacity,
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

//! A Lambda function to get a events of a token.
//!
//! To work, this lambda expects two query string parameters:
//!   * address: Contract address of the collection, in hexadecimal.
//!   * id: The id of the token, in decimal. (TODO: revise the standard we want here).
//!
//! `https://.../token-events?address=0x1234&id=1234`
//!
use ark_dynamodb::{
    event::{ArkEventProvider, DynamoDbEventProvider},
    init_aws_dynamo_client, Client as DynamoClient,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common as common;

/// A struct to bundle all init required by the lambda.
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

async fn function_handler<P: ArkEventProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    event: Request,
) -> Result<Response<Body>, Error> {
    let address = match common::get_query_string_hex_param(&event, "address") {
        Ok(a) => a,
        Err(e) => return e.try_into(),
    };

    let token_id = match common::get_query_string_param(&event, "id") {
        Ok(t) => t,
        Err(e) => return e.try_into(),
    };

    let events = ctx
        .provider
        .get_token_events(&ctx.client, &address, &token_id)
        .await?;

    common::ok_body_rsp(&events)
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
        provider: DynamoDbEventProvider::new(&table_name),
    };

    run(service_fn(|event: Request| async {
        function_handler(&ctx, event).await
    }))
    .await
}

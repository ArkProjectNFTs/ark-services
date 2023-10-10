//! A Lambda function to get all the events for a contract.
//!
//! To work, this lambda expects the following path:
//!     `../events/{contract_address}`
//!
//! where:
//!   * contract_address: Contract address of the collection, in hexadecimal.
//!
//! Examples:
//! `https://.../events/0x1234`
//!
use ark_dynamodb::{
    init_aws_dynamo_client,
    providers::{ArkEventProvider, DynamoDbEventProvider},
    Client as DynamoClient,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, HttpParamSource};

/// A struct to bundle all init required by the lambda.
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

async fn function_handler<P: ArkEventProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    event: Request,
) -> Result<Response<Body>, Error> {
    let address = match common::require_hex_param(&event, "contract_address", HttpParamSource::Path)
    {
        Ok(a) => a,
        Err(e) => return e.try_into(),
    };

    let events = ctx
        .provider
        .get_contract_events(&ctx.client, &address)
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

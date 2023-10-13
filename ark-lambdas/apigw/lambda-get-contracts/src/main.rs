//! A Lambda function to get all contracts that has been identified as collections.
//!
//! To work, this lambda expects the following path:
//!     `../collections`
//!
use ark_dynamodb::{
    init_aws_dynamo_client,
    providers::{ArkContractProvider, DynamoDbContractProvider},
    Client as DynamoClient, DynamoDbOutput,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, ArkApiResponse};

/// A struct to bundle all init required by the lambda.
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

async fn function_handler<P: ArkContractProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    _event: Request,
) -> Result<Response<Body>, Error> {
    let rsp = ctx.provider.get_contracts(&ctx.client).await?;

    // TODO: store in cache the LEK if not none + get the hash.

    common::ok_body_rsp(&ArkApiResponse {
        cursor: rsp.lek,
        result: rsp.inner(),
    })
    // match ctx.provider.get_contracts(&ctx.client).await {
    //     Ok(data) => common::ok_body_rsp(&data),
    //     Err(e) => {
    //         println!("{:?}", e);
    //         common::internal_server_error_rsp(&e.to_string())
    //     }
    // }
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
    let limit = Some(100);

    let ctx = Ctx {
        client: init_aws_dynamo_client().await,
        provider: DynamoDbContractProvider::new(&table_name, limit),
    };

    run(service_fn(|event: Request| async {
        function_handler(&ctx, event).await
    }))
    .await
}

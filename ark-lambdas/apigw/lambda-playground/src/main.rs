#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! A playground to interact with dynamodb locally using providers.
//!
use ark_dynamodb::{
    init_aws_dynamo_client, pagination::DynamoDbPaginator, providers::*, ArkDynamoDbProvider,
    Client as DynamoClient, DynamoDbCtx,
};
use arkproject::metadata::types::TokenMetadata;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError,
};

/// A struct to bundle all init required by the lambda.
struct Ctx {
    db: DynamoDbCtx,
    provider: ArkDynamoDbProvider,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let ctx = LambdaCtx::from_event(&event).await?;

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

    run(service_fn(|event: Request| async {
        function_handler(event).await
    }))
    .await
}

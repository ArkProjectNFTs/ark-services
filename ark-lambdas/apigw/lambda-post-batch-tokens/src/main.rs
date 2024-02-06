use std::f32::consts::E;

use ark_dynamodb::providers::DynamoDbTokenProvider;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, RequestPayloadExt, Response};
use lambda_http_common::{HttpParamSource, LambdaCtx, LambdaHttpError};
use serde::Deserialize;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Deserialize)]
struct Token {
    contract_address: String,
    token_id: String,
}

#[derive(Deserialize)]
struct BodyParameters {
    tokens: Vec<Token>,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let ctx = LambdaCtx::from_event(&event).await?;
    let provider = DynamoDbTokenProvider::new(&ctx.table_name, ctx.max_items_limit);

    // let (address, token_id_hex) = get_params(&event)?;

    let message = format!("Hello this is an AWS Lambda HTTP request");

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

async fn get_params(event: &Request) -> Result<BodyParameters, LambdaHttpError> {
    match event.payload() {
        Ok(p) => {
            // TODO

            return Err(LambdaHttpError::ParamParsing(
                "Failed to parse request body".to_string(),
            ));
        }
        Err(e) => Err(LambdaHttpError::ParamParsing(
            "Failed to parse request body".to_string(),
        )),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

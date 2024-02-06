use ark_dynamodb::providers::{
    token::types::TokensParams, ArkTokenProvider, DynamoDbTokenProvider,
};
use common::{format::hex_or_dec_from_str, LambdaHttpResponse};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, ArkApiResponse, LambdaCtx, LambdaHttpError};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Deserialize, Serialize)]
struct BodyParameters {
    tokens: Vec<TokensParams>,
}

async fn process_event(
    ctx: &LambdaCtx,
    token_params: &Vec<TokensParams>,
) -> Result<LambdaHttpResponse, Error> {
    let provider = DynamoDbTokenProvider::new(&ctx.table_name, ctx.max_items_limit);

    let dynamo_rsp = provider
        .get_batch_tokens(&ctx.dynamodb, token_params)
        .await?;

    let items = dynamo_rsp.inner();
    // let cursor = ctx.paginator.store_cursor(&dynamo_rsp.lek)?;

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor: None,
        total_count: None,
        result: items,
    })?;

    let capacity = dynamo_rsp.consumed_capacity_units.unwrap_or(0.0);

    Ok(LambdaHttpResponse {
        capacity,
        inner: rsp,
    })
}

async fn function_handler(event: Request) -> Result<Response<Body>, LambdaHttpError> {
    let ctx = LambdaCtx::from_event(&event).await?;
    let body_params = get_params(&event).await?;
    let r = process_event(&ctx, &body_params.tokens).await;

    let token_params_str = match serde_json::to_string(&body_params.tokens) {
        Ok(token_params_str) => token_params_str,
        Err(_e) => "".to_string(),
    };

    let mut req_params = HashMap::new();
    req_params.insert("tokens".to_string(), token_params_str);

    match r {
        Ok(lambda_rsp) => {
            // ctx.register_usage(req_params, Some(&lambda_rsp)).await?;
            Ok(lambda_rsp.inner)
        }
        Err(e) => {
            error!("Error processing event: {:?}", e);
            // ctx.register_usage(req_params, None).await?;
            Err(LambdaHttpError::ResponseError)
        }
    }
}

use tracing::error;

async fn get_params(event: &Request) -> Result<BodyParameters, LambdaHttpError> {
    let body = event.body();
    let body_str = match body {
        Body::Text(text) => text,
        _ => {
            error!("Body is not text");
            return Err(LambdaHttpError::ParamMissing(String::from("Body error")));
        }
    };

    let body_params: BodyParameters = serde_json::from_str(body_str).map_err(|e| {
        error!("Error parsing body parameters: {:?}", e);
        LambdaHttpError::ParamParsing(String::from("Body error"))
    })?;

    if body_params.tokens.is_empty() {
        return Err(LambdaHttpError::ParamMissing(String::from(
            "No tokens provided",
        )));
    }

    let results: Result<Vec<TokensParams>, LambdaHttpError> = body_params
        .tokens
        .iter()
        .map(|token_param| {
            let contract_address = token_param.contract_address.to_lowercase();
            hex_or_dec_from_str(&token_param.token_id, "tokens")
                .map(|token_id| TokensParams {
                    contract_address,
                    token_id,
                })
                .map_err(|_e| LambdaHttpError::ParamParsing("Tokens error".into()))
        })
        .collect();

    match results {
        Ok(tokens) => Ok(BodyParameters { tokens }),
        Err(e) => Err(e),
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

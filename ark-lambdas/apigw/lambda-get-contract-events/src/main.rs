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
use ark_dynamodb::providers::{ArkEventProvider, DynamoDbEventProvider};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError, LambdaHttpResponse,
};
use std::collections::HashMap;
use tracing::{debug, error, info};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    info!("Get contract events: {:?}", event);

    // 1. Init the context.
    let ctx = LambdaCtx::from_event(&event).await?;

    // 2. Get params.
    let address = get_params(&event)?;

    info!("Address: {:?}", address);

    // 3. Process the request.
    let r = process_event(&ctx, &address).await;

    // 4. Send the response.
    let mut req_params = HashMap::new();
    req_params.insert("address".to_string(), address.clone());

    match r {
        Ok(lambda_rsp) => Ok(lambda_rsp.inner),
        Err(e) => {
            error!("Error: {:?}", e);
            Err(e)
        }
    }
}

async fn process_event(ctx: &LambdaCtx, address: &str) -> Result<LambdaHttpResponse, Error> {
    info!(
        "Processing event... table_name={}, max_items_limit={:?}",
        ctx.table_name, ctx.max_items_limit
    );

    let provider = DynamoDbEventProvider::new(&ctx.table_name, ctx.max_items_limit);
    let dynamo_rsp = provider.get_contract_events(&ctx.dynamodb, address).await?;

    debug!("DynamoDB response: {:?}", dynamo_rsp);

    let items = dynamo_rsp.inner();
    let last_evaluated_key = &dynamo_rsp.lek;

    debug!("Last evaluated key: {:?}", last_evaluated_key);

    let cursor = ctx.paginator.store_cursor(&dynamo_rsp.lek)?;

    debug!("Cursor: {:?}", cursor);

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor,
        total_count: None,
        result: items,
    })?;

    debug!("Response: {:?}", rsp);

    let capacity = dynamo_rsp.consumed_capacity_units.unwrap_or(0.0);

    Ok(LambdaHttpResponse {
        capacity,
        inner: rsp,
    })
}

fn get_params(event: &Request) -> Result<String, LambdaHttpError> {
    common::require_hex_param(event, "contract_address", HttpParamSource::Path)
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

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::RequestExt;
    use lambda_http_common::format::pad_hex;
    use std::collections::HashMap;

    #[tokio::test]
    async fn params_ok() {
        let address = "0x1234".to_string();

        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), address.clone());

        let req = Request::default().with_path_parameters(params.clone());

        let address = get_params(&req).unwrap();

        assert_eq!(address, pad_hex(&address));
    }

    #[tokio::test]
    async fn parmas_bad_hexadecimal_address() {
        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), "1234".to_string());

        let req = Request::default().with_path_parameters(params.clone());

        match get_params(&req) {
            Ok(_) => panic!("expecting error"),
            Err(e) => match e {
                LambdaHttpError::ParamParsing(s) => {
                    assert_eq!(
                        s,
                        "Param contract_address is expected to be hexadecimal string"
                    )
                }
                _ => panic!("expected ParamParsing"),
            },
        }
    }

    #[tokio::test]
    async fn params_missing_address() {
        let mut params = HashMap::new();
        params.insert("blabla".to_string(), "1".to_string());

        let req = Request::default().with_path_parameters(params.clone());

        match get_params(&req) {
            Ok(_) => panic!("expecting error"),
            Err(e) => match e {
                LambdaHttpError::ParamMissing(s) => {
                    assert_eq!(s, "Param contract_address is missing")
                }
                _ => panic!("expected ParamMissing"),
            },
        }
    }
}

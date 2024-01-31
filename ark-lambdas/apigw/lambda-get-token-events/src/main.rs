//! A Lambda function to get all the events for a token.
//!
//! To work, this lambda expects the following path:
//!     `../events/{contract_address}/{token_id}`
//!
//! where:
//!   * contract_address: Contract address of the collection, in hexadecimal.
//!   * token_id: The id of the token, in hexadecimal or decimal.
//!
//! Examples:
//! `https://.../events/0x1234/1`
//! `https://.../events/0x1234/0x1`
//!
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
    let (address, token_id_hex) = get_params(&event)?;

    // 3. Process the request.
    let r = process_event(&ctx, &address, &token_id_hex).await;

    // 4. Send the response.
    let mut req_params = HashMap::new();
    req_params.insert("address".to_string(), address.clone());
    req_params.insert("token_id_hex".to_string(), token_id_hex.clone());

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

async fn process_event(
    ctx: &LambdaCtx,
    address: &str,
    token_id_hex: &str,
) -> Result<LambdaHttpResponse, Error> {
    let provider = DynamoDbEventProvider::new(&ctx.table_name, ctx.max_items_limit);

    let dynamo_rsp = provider
        .get_token_events(&ctx.dynamodb, address, token_id_hex)
        .await?;

    let items = dynamo_rsp.inner();
    let cursor = ctx.paginator.store_cursor(&dynamo_rsp.lek)?;

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor,
        total_count: None,
        result: items,
    })?;

    let capacity = dynamo_rsp.consumed_capacity_units.unwrap_or(0.0);

    Ok(LambdaHttpResponse {
        capacity,
        inner: rsp,
    })
}

fn get_params(event: &Request) -> Result<(String, String), LambdaHttpError> {
    let address = match common::require_hex_param(event, "contract_address", HttpParamSource::Path)
    {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    let token_id_hex =
        match common::require_hex_or_dec_param(event, "token_id", HttpParamSource::Path) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

    Ok((address, token_id_hex))
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
        let token_id = "1".to_string();

        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), address.clone());
        params.insert("token_id".to_string(), token_id.clone());

        let req = Request::default().with_path_parameters(params.clone());

        let (address, token_id_hex) = get_params(&req).unwrap();

        assert_eq!(address, pad_hex(&address));
        assert_eq!(token_id_hex, pad_hex(&token_id));
    }

    #[tokio::test]
    async fn parmas_bad_hexadecimal_address() {
        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), "1234".to_string());
        params.insert("token_id".to_string(), "1".to_string());

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
        params.insert("token_id".to_string(), "1".to_string());

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

    #[tokio::test]
    async fn params_missing_token_id() {
        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), "0x1".to_string());

        let req = Request::default().with_path_parameters(params.clone());

        match get_params(&req) {
            Ok(_) => panic!("expecting error"),
            Err(e) => match e {
                LambdaHttpError::ParamMissing(s) => {
                    assert_eq!(s, "Param token_id is missing")
                }
                _ => panic!("expected ParamMissing"),
            },
        }
    }

    #[tokio::test]
    async fn parmas_bad_token_id() {
        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), "0x1234".to_string());
        params.insert("token_id".to_string(), "a bad token".to_string());

        let req = Request::default().with_path_parameters(params.clone());

        match get_params(&req) {
            Ok(_) => panic!("expecting error"),
            Err(e) => match e {
                LambdaHttpError::ParamParsing(s) => {
                    assert_eq!(s, "Param token_id out of range decimal value")
                }
                _ => panic!("expected ParamParsing"),
            },
        }
    }
}

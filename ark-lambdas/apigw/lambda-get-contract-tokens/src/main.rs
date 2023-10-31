//! A Lambda function to get all the tokens of a contract.
//!
//! To work, this lambda expects the following path:
//!     `../tokens/{contract_address}`
//!
//! where:
//!   * contract_address: Contract address of the collection, in hexadecimal.
//!
//! Examples:
//! `https://.../tokens/0x1234`
//!
use ark_dynamodb::providers::{ArkTokenProvider, DynamoDbTokenProvider};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError, LambdaHttpResponse,
};
use std::collections::HashMap;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // 1. Init the context.
    let ctx = LambdaCtx::from_event(&event).await?;

    // 2. Get params.
    let address = get_params(&event)?;
    let tokens_ids = get_tokens_ids(&event)?;

    // 3. Process the request.
    let r = process_event(&ctx, &address, &tokens_ids).await;

    // 4. Send the response.
    let mut req_params = HashMap::new();
    req_params.insert("address".to_string(), address.clone());
    req_params.insert("tokens_ids".to_string(), tokens_ids.clone().join(", "));

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
    tokens_ids: &[String],
) -> Result<LambdaHttpResponse, Error> {
    let provider = DynamoDbTokenProvider::new(&ctx.table_name, ctx.max_items_limit);

    let dynamo_rsp = provider
        .get_contract_tokens(&ctx.db, address, tokens_ids)
        .await?;

    let items = dynamo_rsp.inner();
    let cursor = ctx.paginator.store_cursor(&dynamo_rsp.lek)?;

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor,
        result: items,
    })?;

    Ok(LambdaHttpResponse {
        capacity: dynamo_rsp.capacity,
        inner: rsp,
    })
}

fn get_params(event: &Request) -> Result<String, LambdaHttpError> {
    common::require_hex_param(event, "contract_address", HttpParamSource::Path)
}

fn get_tokens_ids(event: &Request) -> Result<Vec<String>, LambdaHttpError> {
    let param_name = "tokens_ids";
    let params = event.query_string_parameters_ref();

    if let Some(prs) = params {
        let tokens_ids: Vec<String> = prs
            .all(param_name)
            .unwrap_or(vec![])
            .into_iter()
            .map(|v| v.to_string())
            .collect();

        let mut out = vec![];
        for t_id in tokens_ids {
            out.push(common::format::hex_or_dec_from_str(&t_id, param_name)?);
        }

        Ok(out)
    } else {
        Ok(vec![])
    }
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

    #[test]
    fn params_ok() {
        let address = "0x1234".to_string();

        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), address.clone());

        let req = Request::default().with_path_parameters(params.clone());

        let address = get_params(&req).unwrap();

        assert_eq!(address, pad_hex(&address));
    }

    #[test]
    fn parmas_bad_hexadecimal_address() {
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

    #[test]
    fn params_missing_address() {
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

    #[test]
    fn params_tokens_ids() {
        let mut params = HashMap::new();
        params.insert(
            "tokens_ids".to_string(),
            vec!["0x123".to_string(), "0x77".to_string(), "255".to_string()],
        );

        let req = Request::default().with_query_string_parameters(params.clone());

        let ids = get_tokens_ids(&req).unwrap();
        assert_eq!(ids[0], pad_hex("0x123"));
        assert_eq!(ids[1], pad_hex("0x77"));
        assert_eq!(ids[2], pad_hex("0xff"));
    }
}

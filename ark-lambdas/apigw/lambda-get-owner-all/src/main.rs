use ark_dynamodb::providers::{ArkTokenProvider, DynamoDbTokenProvider};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError, LambdaHttpResponse,
};
use std::collections::HashMap;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // 1. Init the context.
    let ctx = LambdaCtx::from_event(&event).await?;

    // 2. Get params.
    let owner_address = get_params(&event)?;

    // 3. Process the request.
    let r = process_event(&ctx, &owner_address).await;

    // 4. Send the response.
    let mut req_params = HashMap::new();
    req_params.insert("address".to_string(), owner_address.clone());

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

async fn process_event(ctx: &LambdaCtx, owner_address: &str) -> Result<LambdaHttpResponse, Error> {
    let provider = DynamoDbTokenProvider::new(&ctx.table_name, ctx.max_items_limit);

    let contracts_with_items = provider.get_owner_all(&ctx.dynamodb, owner_address).await?;

    let capacity = contracts_with_items
        .consumed_capacity_units
        .unwrap_or_default();

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor: None,
        total_count: None,
        result: contracts_with_items.inner(),
    })?;

    Ok(LambdaHttpResponse {
        capacity,
        inner: rsp,
    })
}

fn get_params(event: &Request) -> Result<String, LambdaHttpError> {
    common::require_hex_param(event, "owner_address", HttpParamSource::Path)
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
        let owner_address = "0x1234".to_string();

        let mut params = HashMap::new();
        params.insert("owner_address".to_string(), owner_address.clone());

        let req = Request::default().with_path_parameters(params.clone());

        let address = get_params(&req).unwrap();

        assert_eq!(address, pad_hex(&owner_address));
    }

    #[tokio::test]
    async fn parmas_bad_hexadecimal_address() {
        let mut params = HashMap::new();
        params.insert("owner_address".to_string(), "1234".to_string());

        let req = Request::default().with_path_parameters(params.clone());

        match get_params(&req) {
            Ok(_) => panic!("expecting error"),
            Err(e) => match e {
                LambdaHttpError::ParamParsing(s) => {
                    assert_eq!(
                        s,
                        "Param owner_address is expected to be hexadecimal string"
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
                    assert_eq!(s, "Param owner_address is missing")
                }
                _ => panic!("expected ParamMissing"),
            },
        }
    }

    #[tokio::test]
    async fn params_address_lowercase() {
        let mut params = HashMap::new();
        params.insert(
            "owner_address".to_string(),
            "0x00A3244a4d2C7C69C70951A003eBA5c32707Cef3CdfB6B27cA63582f51aee078".to_string(),
        );

        let req = Request::default().with_path_parameters(params.clone());

        let address = get_params(&req).unwrap();
        assert_eq!(
            address,
            "0x00a3244a4d2c7c69c70951a003eba5c32707cef3cdfb6b27ca63582f51aee078"
        );
    }
}

//! A Lambda function to get all the tokens of a owner.
//!
//! To work, this lambda expects the following path:
//!     `../owners/{owner_address}/tokens`
//!
//! where:
//!   * owner_address: Contract address of the account contract (owner), in hexadecimal.
//!
//! Examples:
//! `https://.../owners/0x1234/tokens`
//!
use ark_dynamodb::providers::{ArkTokenProvider, DynamoDbTokenProvider};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError, LambdaHttpResponse,
};
use std::collections::HashMap;
use tracing::{error, info};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    info!("Get owner tokens: {:?}", event);

    // 1. Init the context.
    let ctx = LambdaCtx::from_event(&event).await?;

    // 2. Get params.
    let (owner_address, contract_address) = get_params(&event)?;

    info!("Owner address: {:?}", owner_address);
    info!("Contract address: {:?}", contract_address);

    // 3. Process the request.
    let r = process_event(&ctx, &owner_address, contract_address.clone()).await;

    // 4. Send the response.
    let mut req_params = HashMap::new();
    req_params.insert("address".to_string(), owner_address.clone());
    if let Some(caddr) = contract_address {
        req_params.insert("contract_address".to_string(), caddr);
    }

    match r {
        Ok(lambda_rsp) => {
            ctx.register_usage(req_params, Some(&lambda_rsp)).await?;
            Ok(lambda_rsp.inner)
        }
        Err(e) => {
            error!("Error: {:?}", e);
            ctx.register_usage(req_params, None).await?;
            Err(e)
        }
    }
}

async fn process_event(
    ctx: &LambdaCtx,
    owner_address: &str,
    contract_address: Option<String>,
) -> Result<LambdaHttpResponse, Error> {
    info!("Processing event...");

    let provider = DynamoDbTokenProvider::new(&ctx.table_name, ctx.max_items_limit);

    let dynamo_rsp = provider
        .get_owner_tokens(&ctx.dynamodb, owner_address, contract_address)
        .await?;

    let items = dynamo_rsp.inner();
    let last_evaluated_key = &dynamo_rsp.lek;

    info!("Last evaluated key: {:?}", last_evaluated_key);

    let cursor = ctx.paginator.store_cursor(last_evaluated_key)?;

    info!("Cursor: {:?}", cursor);

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor,
        total_count: dynamo_rsp.total_count,
        result: items,
    })?;

    info!("Response: {:?}", rsp);

    let capacity = dynamo_rsp.consumed_capacity_units.unwrap_or(0.0);

    Ok(LambdaHttpResponse {
        capacity,
        inner: rsp,
    })
}

fn get_params(event: &Request) -> Result<(String, Option<String>), LambdaHttpError> {
    let owner_address = common::require_hex_param(event, "owner_address", HttpParamSource::Path)?;

    let contract_address = if let Some(prs) = event.query_string_parameters_ref() {
        if let Some(ca) = prs.first("contract_address") {
            Some(common::format::hex_from_str(ca, "contract_address")?)
        } else {
            None
        }
    } else {
        None
    };

    Ok((owner_address, contract_address))
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

        let (address, _) = get_params(&req).unwrap();

        assert_eq!(address, pad_hex(&owner_address));
    }

    #[tokio::test]
    async fn params_ok_with_optional() {
        let owner_address = "0x1234".to_string();
        let contract_address = "0x11".to_string();

        let mut params = HashMap::new();
        params.insert("owner_address".to_string(), owner_address.clone());

        let mut qparams = HashMap::new();
        qparams.insert("contract_address".to_string(), contract_address.clone());

        let req = Request::default()
            .with_path_parameters(params.clone())
            .with_query_string_parameters(qparams.clone());

        let (address, contract) = get_params(&req).unwrap();

        assert_eq!(address, pad_hex(&owner_address));
        assert_eq!(contract, Some(pad_hex(&contract_address)));
    }

    #[tokio::test]
    async fn params_bad_contract_address_optional() {
        let owner_address = "0x1234".to_string();
        let contract_address = "ahfieh".to_string();

        let mut params = HashMap::new();
        params.insert("owner_address".to_string(), owner_address.clone());

        let mut qparams = HashMap::new();
        qparams.insert("contract_address".to_string(), contract_address.clone());

        let req = Request::default()
            .with_path_parameters(params.clone())
            .with_query_string_parameters(qparams.clone());

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

        let (address, _) = get_params(&req).unwrap();
        assert_eq!(
            address,
            "0x00a3244a4d2c7c69c70951a003eba5c32707cef3cdfb6b27ca63582f51aee078"
        );
    }
}

//! A Lambda function to get all the contracts for which an owner has at least one token.
//!
//! To work, this lambda expects the following path:
//!     `../owners/{owner_address}/contracts`
//!
//! where:
//!   * owner_address: Contract address of the account contract (owner), in hexadecimal.
//!
//! Examples:
//! `https://.../owners/0x1234/contracts`
//!
use ark_dynamodb::providers::{
    ArkContractProvider, ArkTokenProvider, DynamoDbContractProvider, DynamoDbTokenProvider,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError, LambdaHttpResponse,
};
use std::collections::{HashMap, HashSet};

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
    let contract_provider = DynamoDbContractProvider::new(&ctx.table_name, ctx.max_items_limit);

    // Fetch all the tokens and keep unique contracts addresses.
    let mut contract_addresses: HashSet<String> = HashSet::new();
    let mut contracts = vec![];

    let mut capacity = 0.0;

    loop {
        let dynamo_rsp = provider
            .get_owner_tokens(&ctx.dynamodb, owner_address, None)
            .await?;

        for data in dynamo_rsp.inner() {
            if contract_addresses.insert(data.contract_address.clone()) {
                // Was inserted, fetch data of this contract.
                let c_rsp = contract_provider
                    .get_contract(&ctx.dynamodb, &data.contract_address)
                    .await?;

                capacity += c_rsp.capacity;

                if let Some(contract) = c_rsp.inner().clone() {
                    contracts.push(contract);
                }
            }
        }

        capacity += dynamo_rsp.capacity;

        if dynamo_rsp.lek.is_none() {
            break;
        }
    }

    let rsp = common::ok_body_rsp(&ArkApiResponse {
        cursor: None,
        result: contracts,
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

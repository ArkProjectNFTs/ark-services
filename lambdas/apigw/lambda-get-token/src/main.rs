//! A Lambda function to get a token..
//!
//! To work, this lambda expects two query string parameters:
//!   * address: Contract address of the collection, in hexadecimal.
//!   * id: The id of the token, in decimal. (TODO: revise the standard we want here).
//!
//! `https://.../token?address=0x1234&id=1234`
//!
use ark_dynamodb::{
    init_aws_dynamo_client,
    token::{ArkTokenProvider, DynamoDbTokenProvider},
    Client as DynamoClient,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common as common;

/// A struct to bundle all init required by the lambda.
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

async fn function_handler<P: ArkTokenProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    event: Request,
) -> Result<Response<Body>, Error> {
    let address = match common::get_query_string_hex_param(&event, "address") {
        Ok(a) => a,
        Err(e) => return e.try_into(),
    };

    let token_id = match common::get_query_string_param(&event, "id") {
        Ok(t) => t,
        Err(e) => return e.try_into(),
    };

    if let Some(data) = ctx
        .provider
        .get_token(&ctx.client, &address, &token_id)
        .await?
    {
        common::ok_body_rsp(&data)
    } else {
        common::not_found_rsp()
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

    let table_name = std::env::var("ARK_TABLE_NAME").expect("ARK_TABLE_NAME must be set");

    let ctx = Ctx {
        client: init_aws_dynamo_client().await,
        provider: DynamoDbTokenProvider::new(&table_name),
    };

    run(service_fn(|event: Request| async {
        function_handler(&ctx, event).await
    }))
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_dynamo_common::{
        init_aws_dynamo_client,
        token::{MockArkTokenProvider, TokenData},
    };
    use lambda_http::{Body, RequestExt};

    use std::collections::HashMap;

    async fn get_mock_ctx() -> Ctx<MockArkTokenProvider> {
        Ctx {
            client: init_aws_dynamo_client().await,
            provider: MockArkTokenProvider::default(),
        }
    }

    #[tokio::test]
    async fn request_ok() {
        let address = "0x1234".to_string();
        let token_id = "1".to_string();

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.clone());
        params.insert("id".to_string(), token_id.clone());

        let req = Request::default().with_query_string_parameters(params.clone());

        let mut ctx = get_mock_ctx().await;
        ctx.provider.expect_get_token().returning(move |_, _, _| {
            Ok(Some(TokenData {
                block_number: 123,
                mint_timestamp: 8888,
                mint_address: "0x1111".to_string(),
                owner: "0x2222".to_string(),
                token_id: token_id.clone(),
                contract_address: "0x3333".to_string(),
            }))
        });

        let rsp = function_handler(&ctx, req)
            .await
            .expect("failed to handle request");

        assert_eq!(rsp.status(), 200);
    }

    #[tokio::test]
    async fn bad_hexadecimal_address() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "1234".to_string());
        params.insert("id".to_string(), "1".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        // No setup, as the lambda will return an error before any dynamodb stuff.
        let rsp = function_handler(&get_mock_ctx().await, req)
            .await
            .expect("failed to handle request");

        assert_eq!(rsp.status(), 400);

        let body = match rsp.body() {
            Body::Text(t) => t,
            _ => panic!("Body is expected to be a string"),
        };

        assert_eq!(body, "Param address is expected to be hexadecimal string");
    }

    #[tokio::test]
    async fn missing_address() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "1".to_string());

        let req = Request::default().with_query_string_parameters(params.clone());

        // No setup, as the lambda will return an error before any dynamodb stuff.
        let rsp = function_handler(&get_mock_ctx().await, req)
            .await
            .expect("failed to handle request");

        assert_eq!(rsp.status(), 400);

        let body = match rsp.body() {
            Body::Text(t) => t,
            _ => panic!("Body is expected to be a string"),
        };

        assert_eq!(body, "Param address is missing");
    }
}

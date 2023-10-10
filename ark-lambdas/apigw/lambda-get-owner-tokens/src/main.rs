//! A Lambda function to get all the tokens of a owner.
//!
//! To work, this lambda expects the following path:
//!     `../tokens/owner/{owner_address}`
//!
//! where:
//!   * owner_address: Contract address of the account contract (owner), in hexadecimal.
//!
//! Examples:
//! `https://.../tokens/owner/0x1234`
//!
use ark_dynamodb::{
    init_aws_dynamo_client,
    providers::{ArkTokenProvider, DynamoDbTokenProvider},
    Client as DynamoClient,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, HttpParamSource};

/// A struct to bundle all init required by the lambda.
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

async fn function_handler<P: ArkTokenProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    event: Request,
) -> Result<Response<Body>, Error> {
    let address = match common::require_hex_param(&event, "owner_address", HttpParamSource::Path) {
        Ok(a) => a,
        Err(e) => return e.try_into(),
    };

    match ctx.provider.get_owner_tokens(&ctx.client, &address).await {
        Ok(data) => common::ok_body_rsp(&data),
        Err(e) => {
            println!("{:?}", e);
            common::internal_server_error_rsp(&e.to_string())
        }
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
    use ark_dynamodb::{init_aws_dynamo_client, providers::token::MockArkTokenProvider};
    use arkproject::pontos::storage::types::TokenInfo;
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

        let mut params = HashMap::new();
        params.insert("owner_address".to_string(), address.clone());

        let req = Request::default().with_path_parameters(params.clone());

        let mut ctx = get_mock_ctx().await;
        ctx.provider
            .expect_get_contract_tokens()
            .returning(move |_, _| {
                Ok(vec![TokenInfo {
                    mint_block_number: Some(123),
                    mint_timestamp: Some(8888),
                    mint_address: Some("0x1111".to_string()),
                    owner: "0x2222".to_string(),
                    token_id: "1234".to_string(),
                    contract_address: "0x3333".to_string(),
                    ..Default::default()
                }])
            });

        let rsp = function_handler(&ctx, req)
            .await
            .expect("failed to handle request");

        assert_eq!(rsp.status(), 200);
    }

    #[tokio::test]
    async fn bad_hexadecimal_address() {
        let mut params = HashMap::new();
        params.insert("owner_address".to_string(), "1234".to_string());
        let req = Request::default().with_path_parameters(params.clone());

        // No setup, as the lambda will return an error before any dynamodb stuff.
        let rsp = function_handler(&get_mock_ctx().await, req)
            .await
            .expect("failed to handle request");

        assert_eq!(rsp.status(), 400);

        let body = match rsp.body() {
            Body::Text(t) => t,
            _ => panic!("Body is expected to be a string"),
        };

        assert_eq!(
            body,
            "Param owner_address is expected to be hexadecimal string"
        );
    }

    #[tokio::test]
    async fn missing_address() {
        let mut params = HashMap::new();
        params.insert("token_id".to_string(), "1".to_string());

        let req = Request::default().with_path_parameters(params.clone());

        // No setup, as the lambda will return an error before any dynamodb stuff.
        let rsp = function_handler(&get_mock_ctx().await, req)
            .await
            .expect("failed to handle request");

        assert_eq!(rsp.status(), 400);

        let body = match rsp.body() {
            Body::Text(t) => t,
            _ => panic!("Body is expected to be a string"),
        };

        assert_eq!(body, "Param owner_address is missing");
    }
}

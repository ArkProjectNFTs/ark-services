//! A Lambda function to get a collection from it's address in hexadecimal representation.
//!
//! To work, this lambda expects the collection address as query string parameter "address".
//!
//! `https://.../collection?address=0x1234`
//!
use lambda_dynamo_common::{
    collection::{ArkCollectionProvider, DynamoDbCollectionProvider},
    init_aws_dynamo_client, Client as DynamoClient,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common as common;

/// A struct to bundle all init required by the lambda.
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

async fn function_handler<P: ArkCollectionProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    event: Request,
) -> Result<Response<Body>, Error> {
    let address = if let Some(a) = common::get_query_string_param(&event, "address") {
        if !common::is_hexadecimal_with_prefix(&a) {
            return common::bad_request_rsp("Invalid address");
        } else {
            a
        }
    } else {
        return common::bad_request_rsp("Missing address");
    };

    if let Some(data) = ctx.provider.get_collection(&ctx.client, &address).await? {
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
        provider: DynamoDbCollectionProvider::new(&table_name),
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
        collection::{CollectionData, MockArkCollectionProvider},
        init_aws_dynamo_client,
    };
    use lambda_http::{Body, RequestExt};

    use std::collections::HashMap;

    async fn get_mock_ctx() -> Ctx<MockArkCollectionProvider> {
        Ctx {
            client: init_aws_dynamo_client().await,
            provider: MockArkCollectionProvider::default(),
        }
    }

    #[tokio::test]
    async fn request_ok() {
        let address = "0x1234".to_string();

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.clone());

        let req = Request::default().with_query_string_parameters(params.clone());

        let mut ctx = get_mock_ctx().await;
        ctx.provider.expect_get_collection().returning(move |_, _| {
            Ok(Some(CollectionData {
                block_number: 123,
                contract_type: "erc721".to_string(),
                contract_address: address.clone(),
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
        params.insert("address".to_string(), "contractA".to_string());
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

        assert_eq!(body, "Invalid address");
    }

    #[tokio::test]
    async fn missing_address() {
        let req = Request::default();

        // No setup, as the lambda will return an error before any dynamodb stuff.
        let rsp = function_handler(&get_mock_ctx().await, req)
            .await
            .expect("failed to handle request");

        assert_eq!(rsp.status(), 400);

        let body = match rsp.body() {
            Body::Text(t) => t,
            _ => panic!("Body is expected to be a string"),
        };

        assert_eq!(body, "Missing address");
    }
}

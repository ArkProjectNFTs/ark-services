#![allow(dead_code)]

//! A playground to interact with dynamodb locally using providers.
//!
use ark_dynamodb::{
    init_aws_dynamo_client, pagination::DynamoDbPaginator, providers::*, ArkDynamoDbProvider,
    Client as DynamoClient, DynamoDbCtx,
};
use arkproject::metadata::types::TokenMetadata;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common as common;

/// A struct to bundle all init required by the lambda.
struct Ctx {
    db: DynamoDbCtx,
    provider: ArkDynamoDbProvider,
}

async fn function_handler(ctx: &Ctx, _event: Request) -> Result<Response<Body>, Error> {
    let paginator = DynamoDbPaginator::new("redis://127.0.0.1:6379");

    let lek = paginator.get_cursor("06096071-21df-4c6b-8517-dc1c4bc47751")?;
    println!("LEK: {:?}", lek);

    match ctx.provider.contract.get_contracts(&ctx.db).await {
        Ok(r) => {
            println!("RES____ |||| {:?}", r);
            match paginator.store_cursor(r.lek) {
                Ok(h) => println!("CURSOR HASH {:?}", h),
                Err(e) => println!("ERRR___ {:?}", e),
            }
        }
        Err(e) => println!("ERR____ {:?}", e),
    };

    // let md = TokenMetadata {
    //     ..Default::default()
    // };
    // match ctx
    //     .provider
    //     .token
    //     .update_metadata(
    //         &ctx.client,
    //         "0x05004ab1e4f512e43f46311580dc4a0a053f146310c622344dfddab8fed7d5b0",
    //         "0x00000000000000000000000000000000000000000000000000000000000001a4",
    //         &md,
    //     )
    //     .await
    // {
    //     Ok(_) => (),
    //     Err(e) => println!("ERR____ {:?}", e),
    // };

    // match ctx.provider.block.clean(&ctx.client, 1694176591, None).await {
    //     Ok(_) => (),
    //     Err(e) => println!("ERR____ {:?}", e),
    // };

    common::ok_body_rsp(&1)
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
    let limit = Some(1);

    let client = init_aws_dynamo_client().await;
    let db = DynamoDbCtx {
        client,
        exclusive_start_key: None,
        api_key: "TODO_FROM_EVENT".to_string(),
    };

    let ctx = Ctx {
        db,
        provider: ArkDynamoDbProvider::new(&table_name, limit),
    };

    run(service_fn(|event: Request| async {
        function_handler(ctx, event).await
    }))
    .await
}

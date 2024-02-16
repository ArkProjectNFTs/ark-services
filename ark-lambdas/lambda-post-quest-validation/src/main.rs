use ark_sqlx::providers::quests::{QuestProvider, QuestToValidate};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Database connection error");

    let data: QuestToValidate = serde_json::from_slice(event.body().as_ref())
        .map_err(|_| lambda_http::Error::from("Error parsing request body"))?;

    QuestProvider::validate(&db_pool, &data)
        .await
        .map_err(|_| lambda_http::Error::from("Unable to validate quest"))?;

    let response: Response<Body> = common::ok_body_rsp(&"Quest registered successfully")
        .map_err(|e| lambda_http::Error::from(e.to_string()))?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

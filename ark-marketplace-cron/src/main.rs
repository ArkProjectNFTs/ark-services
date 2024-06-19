mod db;
mod tasks;

use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;
use tasks::tokens::update_listed_tokens;
use db::get_db_pool;


#[tokio::main]
async fn main()  -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_logging();
    let pool = get_db_pool().await.expect("Failed to get database pool");

    // @todo when adding new calculation add spawn & try_join!
    update_listed_tokens(&pool).await;

    Ok(())
}

fn init_logging() {
    const DEFAULT_LOG_FILTER: &str = "trace";
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .or(EnvFilter::try_new(DEFAULT_LOG_FILTER))
                    .expect("Invalid RUST_LOG filters"),
            )
            .finish(),
    )
    .expect("Failed to set the global tracing subscriber");
}

//! How to use Diri library.
//!
//! Can be run with `cargo run --example diri`.
//!
use anyhow::Result;
use ark_sqlx::providers::SqlxArkchainProvider;
use arkproject::diri::{event_handler::EventHandler, Diri};
use async_trait::async_trait;
use starknet::{
    core::types::BlockId,
    providers::{jsonrpc::HttpTransport, AnyProvider, JsonRpcClient, Provider},
};
use std::sync::Arc;
use tracing::{error, info, trace, warn};
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    trace!("Starting...");

    let rpc_url = Url::parse("http://127.0.0.1:7777").unwrap();
    let provider = Arc::new(AnyProvider::JsonRpcHttp(JsonRpcClient::new(
        HttpTransport::new(rpc_url),
    )));

    // Quick launch locally:
    // sudo docker run -d --name arkchain-db -p 5432:5432 -e POSTGRES_PASSWORD=123 postgres
    // sqlx database reset --database-url postgres://postgres:123@localhost:5432/arkchain
    let storage =
        SqlxArkchainProvider::new("postgres://postgres:123@localhost:5432/arkchain").await?;
    let handler = DefaultEventHandler {};

    let indexer = Arc::new(Diri::new(
        provider.clone(),
        Arc::new(storage),
        Arc::new(handler),
    ));

    let sleep_secs = 1;
    let mut from = 0;
    let range = 1;

    // Set to None to keep polling the head of chain.
    let to = None;

    info!(
        "Starting arkchain indexer: from:{} to:{:?} range:{}",
        from, to, range,
    );

    loop {
        let latest_block = match provider.block_number().await {
            Ok(block_number) => block_number,
            Err(e) => {
                error!("Can't get arkchain block number: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)).await;
                continue;
            }
        };

        trace!("Latest block {latest_block} (from={from})");

        let start = from;
        let mut end = std::cmp::min(from + range, latest_block);
        if let Some(to) = to {
            if end > to {
                end = to
            }
        }

        if start > end {
            trace!("Nothing to fetch at block {start}");
            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)).await;
            continue;
        }

        trace!("Fetching blocks {start} - {end}");
        match indexer
            .index_block_range(BlockId::Number(start), BlockId::Number(end))
            .await
        {
            Ok(_) => {
                trace!("Blocks successfully indexed");

                if let Some(to) = to {
                    if end >= to {
                        trace!("`to` block was reached, exit.");
                        return Ok(());
                    }
                }

                // +1 to not re-index the end block.
                from = end + 1;
            }
            Err(e) => {
                error!("Blocks indexing error: {}", e);

                // TODO: for now, any failure on the block range, we skip it.
                // Can be changed as needed.
                warn!("Skipping blocks range: {} - {}", start, end);
                from = end + 1;
            }
        };

        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)).await;
    }
}

/// Initializes the logging, ensuring that the `RUST_LOG` environment
/// variable is always considered first.
fn init_logging() {
    const DEFAULT_LOG_FILTER: &str = "info,diri=trace,ark=trace";

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

// Default event hanlder.
struct DefaultEventHandler;

#[async_trait]
impl EventHandler for DefaultEventHandler {
    async fn on_block_processed(&self, block_number: u64) {
        println!("event: block processed {:?}", block_number);
    }
}

use anyhow::Result;
use ark_sqlx::providers::SqlxMarketplaceProvider;
use arkproject::diri::{event_handler::EventHandler, Diri};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use dotenv::dotenv;
use serde::Deserialize;
use starknet::{
    core::types::BlockId,
    providers::{jsonrpc::HttpTransport, AnyProvider, JsonRpcClient, Provider},
};
use std::{env, sync::Arc, fs, path::Path};
use tracing::{error, info, trace, warn};
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;
use url::Url;

#[derive(Deserialize)]
struct DatabaseCredentials {
    username: String,
    password: String,
    dbname: String,
    port: u16,
    host: String,
}

#[derive(Debug)]
struct Checkpoint {
    file_path: String,
    current_block: u64,
}

impl Checkpoint {
    async fn new(file_path: String) -> Result<Self> {
        let current_block = if Path::new(&file_path).exists() {
            let contents = fs::read_to_string(&file_path)?;
            contents.trim().parse::<u64>()?
        } else {
            fs::write(&file_path, "0")?;
            trace!("Created new checkpoint file at {}", file_path);
            0
        };

        Ok(Checkpoint {
            file_path,
            current_block,
        })
    }

    async fn save(&self, block_number: u64) -> Result<()> {
        fs::write(&self.file_path, block_number.to_string())?;
        trace!("Saved checkpoint: block {}", block_number);
        Ok(())
    }

    fn get_block(&self) -> u64 {
        self.current_block
    }
}

async fn get_database_url() -> Result<String> {
    match std::env::var("DATABASE_URL") {
        Ok(url) => Ok(url),
        Err(_) => {
            let secret_name = std::env::var("AWS_SECRET_NAME").expect("AWS_SECRET_NAME not set");
            let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
            let client = aws_sdk_secretsmanager::Client::new(&config);
            let secret_value = client
                .get_secret_value()
                .secret_id(secret_name)
                .send()
                .await?;
            let result = secret_value.secret_string.unwrap();

            let creds: DatabaseCredentials = serde_json::from_str(&result)?;
            let database_url = format!(
                "postgres://{}:{}@{}:{}/{}",
                creds.username, creds.password, creds.host, creds.port, creds.dbname
            );

            Ok(database_url)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    init_logging();
    trace!("Starting...");

    let rpc_url = env::var("ARKCHAIN_RPC_PROVIDER").expect("ARKCHAIN_RPC_PROVIDER must be set");
    let rpc_url_converted = Url::parse(&rpc_url).unwrap();

    let database_uri = get_database_url().await?;

    let provider = Arc::new(AnyProvider::JsonRpcHttp(JsonRpcClient::new(
        HttpTransport::new(rpc_url_converted.clone()),
    )));

    let storage = SqlxMarketplaceProvider::new(&database_uri).await?;
    let handler = DefaultEventHandler {};

    let indexer = Arc::new(Diri::new(
        provider.clone(),
        Arc::new(storage),
        Arc::new(handler),
    ));

    let sleep_secs = 1;
    let checkpoint = Checkpoint::new("checkpoint.txt".to_string()).await?;
    let mut from = checkpoint.get_block();
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
                        // Save final checkpoint before exiting
                        if let Err(e) = checkpoint.save(end).await {
                            warn!("Failed to save final checkpoint: {}", e);
                        }
                        trace!("`to` block was reached, exit.");
                        return Ok(());
                    }
                }

                // Save the checkpoint after successful indexing
                if let Err(e) = checkpoint.save(end).await {
                    warn!("Failed to save checkpoint: {}", e);
                }

                // +1 to not re-index the end block.
                from = end + 1;
            }
            Err(e) => {
                error!("Blocks indexing error: {}", e);

                // Save checkpoint even on error to track progress
                if let Err(ce) = checkpoint.save(end).await {
                    warn!("Failed to save checkpoint after error: {}", ce);
                }

                warn!("Skipping blocks range: {} - {}", start, end);
                from = end + 1;
            }
        };

        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)).await;
    }
}

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

struct DefaultEventHandler;

#[async_trait]
impl EventHandler for DefaultEventHandler {
    async fn on_block_processed(&self, block_number: u64) {
        println!("event: block processed {:?}", block_number);
    }
}
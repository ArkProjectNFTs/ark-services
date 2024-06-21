mod sana_observer;
use anyhow::Result;
use arkproject::{
    sana::storage::sqlx::MarketplaceSqlxStorage,
    sana::{Sana, SanaConfig},
    starknet::client::{StarknetClient, StarknetClientHttp},
};
use aws_config::BehaviorVersion;
use dotenv::dotenv;
use regex::Regex;
use sana_observer::SanaObserver;
use serde::Deserialize;
use starknet::{
    core::types::{BlockId, BlockTag},
    providers::{jsonrpc::HttpTransport, AnyProvider, JsonRpcClient, Provider},
};
use std::{env, sync::Arc};
use tracing::{error, info, trace, warn};
use tracing_subscriber::{fmt, EnvFilter};
use url::Url;

#[derive(Deserialize)]
struct DatabaseCredentials {
    username: String,
    password: String,
    dbname: String,
    port: u16,
    host: String,
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

    let rpc_url = env::var("RPC_PROVIDER").expect("RPC_PROVIDER must be set");
    let rpc_url_converted = Url::parse(&rpc_url).unwrap();

    let indexer_version = env::var("INDEXER_VERSION").expect("INDEXER_VERSION must be set");
    let indexer_identifier = get_task_id();
    let db_url = get_database_url().await?;
    let chain_id = env::var("CHAIN_ID").expect("CHAIN_ID must be set");
    let force_mode = env::var("FORCE_MODE").is_ok();

    let is_head_of_chain = match std::env::var("HEAD_OF_CHAIN") {
        Ok(val) => val == "true",
        Err(_) => false,
    };

    info!(
        "Starting Indexer. Version={:?}, Identifier={}, Force Mode={}",
        indexer_version, indexer_identifier, force_mode
    );

    let storage = Arc::new(MarketplaceSqlxStorage::new_any(&db_url).await?);

    let starknet_client = Arc::new(StarknetClientHttp::new(rpc_url.as_str())?);

    let sana_observer = Arc::new(SanaObserver::new(
        indexer_version.clone(),
        indexer_identifier.clone(),
    ));

    let provider = Arc::new(AnyProvider::JsonRpcHttp(JsonRpcClient::new(
        HttpTransport::new(rpc_url_converted.clone()),
    )));

    let sana_task = Sana::new(
        Arc::clone(&starknet_client),
        storage,
        Arc::clone(&sana_observer),
        SanaConfig {
            indexer_version,
            indexer_identifier,
        },
    );

    if !is_head_of_chain {
        let from_value = env::var("FROM_BLOCK")
            .ok()
            .and_then(|val| val.parse::<u64>().ok());

        let to_value = env::var("TO_BLOCK")
            .ok()
            .and_then(|val| val.parse::<u64>().ok());

        if let (Some(from_block), Some(to_block)) = (from_value, to_value) {
            match sana_task
                .index_block_range(
                    BlockId::Number(from_block),
                    BlockId::Number(to_block),
                    false,
                    chain_id.as_str(),
                )
                .await
            {
                Ok(_) => {
                    trace!("Blocks successfully indexed");
                    return Ok(());
                }
                Err(e) => {
                    error!("Blocks indexing error: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            error!("FROM_BLOCK or TO_BLOCK environment variable is not set or invalid.");
            return Err(anyhow::anyhow!(
                "FROM_BLOCK or TO_BLOCK environment variable is not set or invalid."
            ));
        }
    }

    let sleep_secs = 1;

    let current_block = match provider.block_number().await {
        Ok(current_block) => current_block,
        Err(e) => {
            error!("Can't get block number {:?}", e);
            0
        }
    };
    let mut from = current_block;
    let range = 1;
    // Set to None to keep polling the head of chain.
    let to = None;

    let mut previous_pending_ts = None;

    trace!("Syncing Sana at head of the chain");
    loop {
        let (pending_ts, _txs) = match starknet_client
            .block_txs_hashes(BlockId::Tag(BlockTag::Pending))
            .await
        {
            Ok((ts, txs)) => (ts, txs),
            Err(e) => {
                error!("Error while fetching pending block txs: {:?}", e);
                continue;
            }
        };
        trace!("Indexing pending block {}...", pending_ts);
        if Some(pending_ts) == previous_pending_ts {
            trace!("Indexing pending block {}...", pending_ts);
            sana_task
                .index_pending_block(pending_ts, chain_id.as_str())
                .await?;
        } else {
            let latest_block = match provider.block_number().await {
                Ok(block_number) => block_number,
                Err(e) => {
                    error!("Can't get block number: {}", e);
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
            match sana_task
                .index_block_range(
                    BlockId::Number(start),
                    BlockId::Number(end),
                    force_mode,
                    chain_id.as_str(),
                )
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
        }

        previous_pending_ts = Some(pending_ts);
        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)).await;
    }
}

fn get_task_id() -> String {
    match env::var("ECS_CONTAINER_METADATA_URI") {
        Ok(container_metadata_uri) => {
            let pattern = Regex::new(r"/v3/([a-f0-9]{32})-").unwrap();
            let task_id = pattern
                .captures(container_metadata_uri.as_str())
                .and_then(|cap| cap.get(1).map(|m| m.as_str()))
                .expect("Can't parse task id from ECS_CONTAINER_METADATA_URI");

            task_id.to_string()
        }
        Err(_) => String::from("LATEST"),
    }
}

fn init_logging() {
    const DEFAULT_LOG_FILTER: &str = "info,sana=trace,ark=trace";

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

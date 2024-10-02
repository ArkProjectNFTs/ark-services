use crate::providers::marketplace::OrderProvider as MarketplaceOrderProvider;
use crate::providers::orderbook::OrderProvider;
use arkproject::diri::storage::types::{
    CancelledData, ExecutedData, FulfilledData, PlacedData, RollbackStatusData,
};
use arkproject::diri::storage::{Storage, StorageError, StorageResult};
use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, Client};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use sqlx::{any::AnyPoolOptions, AnyPool, Error as SqlxError};
use starknet::core::types::{BlockId, BlockTag, Felt, FunctionCall};
use starknet::core::utils::parse_cairo_short_string;
use starknet::macros::selector;
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Provider, Url,
};
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

pub mod marketplace;
pub mod metrics;
pub mod orderbook;

async fn connect_redis() -> Result<Arc<Mutex<MultiplexedConnection>>, Box<dyn Error>> {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL not set");
    let redis_username = std::env::var("REDIS_USERNAME").expect("REDIS_USERNAME not set");
    let redis_password = std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD not set");

    let client = Client::open(format!(
        "redis://{}:{}@{}",
        redis_username, redis_password, redis_url
    ))?;
    let connection = client.get_multiplexed_tokio_connection().await?;
    Ok(Arc::new(Mutex::new(connection)))
}

/// A context for SQLx database.
#[derive(Debug)]
pub struct SqlxCtx {
    pub pool: AnyPool,
}

impl SqlxCtx {
    pub async fn new(db_url: &str) -> Result<Self, ProviderError> {
        sqlx::any::install_default_drivers();

        Ok(Self {
            pool: AnyPoolOptions::new().connect(db_url).await?,
        })
    }
}

#[derive(Debug)]
pub struct SqlxCtxPg {
    pub pool: PgPool,
}

impl SqlxCtxPg {
    pub async fn new(db_url: &str) -> Result<Self, ProviderError> {
        sqlx::any::install_default_drivers();

        Ok(Self {
            pool: PgPoolOptions::new().connect(db_url).await?,
        })
    }
}

/// Generic errors for providers.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Missing data error: {0}")]
    MissingDataError(String),
    #[error("Data value error: {0}")]
    DataValueError(String),
    #[error("Pagination cache error: {0}")]
    PaginationCacheError(String),
    #[error("Parsing error: {0}")]
    ParsingError(String),
}

impl From<SqlxError> for ProviderError {
    fn from(e: SqlxError) -> Self {
        ProviderError::DatabaseError(e.to_string())
    }
}

impl From<&str> for ProviderError {
    fn from(err: &str) -> Self {
        ProviderError::DataValueError(err.to_string())
    }
}

impl From<ProviderError> for StorageError {
    fn from(e: ProviderError) -> Self {
        StorageError::ProviderError(e.to_string())
    }
}

pub struct SqlxArkchainProvider {
    client: SqlxCtx,
}

impl SqlxArkchainProvider {
    pub async fn new(sqlx_conn_str: &str) -> Result<Self, ProviderError> {
        let sqlx = SqlxCtx::new(sqlx_conn_str).await?;

        Ok(Self { client: sqlx })
    }
}

#[async_trait]
impl Storage for SqlxArkchainProvider {
    async fn register_placed(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &PlacedData,
    ) -> StorageResult<()> {
        Ok(OrderProvider::register_placed(&self.client, block_id, block_timestamp, data).await?)
    }

    async fn register_cancelled(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &CancelledData,
    ) -> StorageResult<()> {
        Ok(
            OrderProvider::register_cancelled(&self.client, block_id, block_timestamp, data)
                .await?,
        )
    }

    async fn register_fulfilled(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &FulfilledData,
    ) -> StorageResult<()> {
        Ok(
            OrderProvider::register_fulfilled(&self.client, block_id, block_timestamp, data)
                .await?,
        )
    }

    async fn register_executed(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &ExecutedData,
    ) -> StorageResult<()> {
        Ok(OrderProvider::register_executed(&self.client, block_id, block_timestamp, data).await?)
    }

    async fn status_back_to_open(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &RollbackStatusData,
    ) -> StorageResult<()> {
        Ok(
            OrderProvider::status_back_to_open(&self.client, block_id, block_timestamp, data)
                .await?,
        )
    }
}

pub struct SqlxMarketplaceProvider {
    client: SqlxCtxPg,
    redis_conn: Arc<Mutex<MultiplexedConnection>>,
    provider: JsonRpcClient<HttpTransport>,
}

pub trait ContractProvider {
    fn call_contract_method<'a>(
        &'a self,
        contract_address: Felt,
        selector: Felt,
    ) -> impl Future<Output = Result<String, ProviderError>> + 'a + Send;
}

impl ContractProvider for JsonRpcClient<HttpTransport> {
    fn call_contract_method<'a>(
        &'a self,
        contract_address: Felt,
        selector: Felt,
    ) -> impl Future<Output = Result<String, ProviderError>> + 'a + Send {
        async move {
            let call_result = self
                .call(
                    FunctionCall {
                        contract_address,
                        entry_point_selector: selector,
                        calldata: vec![],
                    },
                    BlockId::Tag(BlockTag::Latest),
                )
                .await
                .map_err(|_| ProviderError::ParsingError("Failed to call contract".to_string()))?;

            let mut property: String;
            if let Some(result) = call_result.first() {
                match parse_cairo_short_string(result) {
                    Ok(value) => {
                        property = value;
                    }
                    Err(_) => {
                        return Err(ProviderError::ParsingError(
                            "Failed to parse short string".to_string(),
                        ));
                    }
                }
            } else {
                return Err(ProviderError::ParsingError(
                    "Failed call_result".to_string(),
                ));
            }

            if selector == selector!("decimals") {
                property = (property
                    .chars()
                    .next()
                    .ok_or_else(|| ProviderError::ParsingError("Empty string".to_string()))?
                    as u32)
                    .to_string();
            }

            Ok(property)
        }
    }
}
impl SqlxMarketplaceProvider {
    pub async fn new(sqlx_conn_str: &str) -> Result<Self, ProviderError> {
        let redis_conn = match connect_redis().await {
            Ok(con) => con,
            Err(e) => {
                error!("Failed to connect to Redis: {}", e);
                return Err(ProviderError::DatabaseError(
                    "Failed to connect to Redis".to_string(),
                ));
            }
        };

        let rpc_url =
            std::env::var("ARKCHAIN_RPC_PROVIDER").expect("ARKCHAIN_RPC_PROVIDER not set");
        let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(&rpc_url).unwrap()));

        let sqlx = SqlxCtxPg::new(sqlx_conn_str).await?;

        Ok(Self {
            client: sqlx,
            redis_conn,
            provider,
        })
    }
}

#[async_trait]
impl Storage for SqlxMarketplaceProvider {
    async fn register_placed(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &PlacedData,
    ) -> StorageResult<()> {
        Ok(MarketplaceOrderProvider::register_placed(
            &self.client,
            self.redis_conn.clone(),
            &self.provider,
            block_id,
            block_timestamp,
            data,
        )
        .await?)
    }

    async fn register_cancelled(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &CancelledData,
    ) -> StorageResult<()> {
        Ok(MarketplaceOrderProvider::register_cancelled(
            &self.client,
            self.redis_conn.clone(),
            block_id,
            block_timestamp,
            data,
        )
        .await?)
    }

    async fn register_fulfilled(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &FulfilledData,
    ) -> StorageResult<()> {
        Ok(MarketplaceOrderProvider::register_fulfilled(
            &self.client,
            self.redis_conn.clone(),
            block_id,
            block_timestamp,
            data,
        )
        .await?)
    }

    async fn register_executed(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &ExecutedData,
    ) -> StorageResult<()> {
        Ok(MarketplaceOrderProvider::register_executed(
            &self.client,
            self.redis_conn.clone(),
            block_id,
            block_timestamp,
            data,
        )
        .await?)
    }

    async fn status_back_to_open(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &RollbackStatusData,
    ) -> StorageResult<()> {
        Ok(MarketplaceOrderProvider::status_back_to_open(
            &self.client,
            block_id,
            block_timestamp,
            data,
        )
        .await?)
    }
}

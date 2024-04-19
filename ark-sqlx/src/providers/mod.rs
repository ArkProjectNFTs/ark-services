use arkproject::diri::storage::types::{
    CancelledData, ExecutedData, FulfilledData, PlacedData, RollbackStatusData,
};
use arkproject::diri::storage::{Storage, StorageError, StorageResult};
use async_trait::async_trait;
use sqlx::{any::AnyPoolOptions, AnyPool, Error as SqlxError};

use crate::providers::orderbook::OrderProvider;
use crate::providers::marketplace::OrderProvider as MarketplaceOrderProvider;

pub mod metrics;
pub mod orderbook;
pub mod marketplace;

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
    client: SqlxCtx,
}

impl SqlxMarketplaceProvider {
    pub async fn new(sqlx_conn_str: &str) -> Result<Self, ProviderError> {
        let sqlx = SqlxCtx::new(sqlx_conn_str).await?;

        Ok(Self { client: sqlx })
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
        Ok(MarketplaceOrderProvider::register_placed(&self.client, block_id, block_timestamp, data).await?)
    }

    async fn register_cancelled(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &CancelledData,
    ) -> StorageResult<()> {
        Ok(
            MarketplaceOrderProvider::register_cancelled(&self.client, block_id, block_timestamp, data)
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
            MarketplaceOrderProvider::register_fulfilled(&self.client, block_id, block_timestamp, data)
                .await?,
        )
    }

    async fn register_executed(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &ExecutedData,
    ) -> StorageResult<()> {
        Ok(MarketplaceOrderProvider::register_executed(&self.client, block_id, block_timestamp, data).await?)
    }

    async fn status_back_to_open(
        &self,
        block_id: u64,
        block_timestamp: u64,
        data: &RollbackStatusData,
    ) -> StorageResult<()> {
        Ok(
            MarketplaceOrderProvider::status_back_to_open(&self.client, block_id, block_timestamp, data)
                .await?,
        )
    }
}

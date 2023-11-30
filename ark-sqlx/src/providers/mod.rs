use arkproject::diri::storage::types::NewOrderData;
use arkproject::diri::storage::{Storage, StorageError, StorageResult};
use async_trait::async_trait;
use sqlx::{any::AnyPoolOptions, AnyPool, Error as SqlxError};

use crate::providers::orderbook::OrderProvider;

pub mod metrics;
pub mod orderbook;

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
    async fn add_new_order(
        &self,
        block_id: u64,
        block_timestamp: u64,
        order: &NewOrderData,
    ) -> StorageResult<()> {
        Ok(OrderProvider::add_new_order(&self.client, block_id, block_timestamp, order).await?)
    }
}

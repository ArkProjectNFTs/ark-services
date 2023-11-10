pub mod metrics;

use sqlx::{any::AnyPoolOptions, AnyPool, Error as SqlxError};

/// A context for SQLx database.
#[derive(Debug)]
pub struct SqlxCtx {
    pub pool: AnyPool,
}

impl SqlxCtx {
    pub async fn new(db_url: &str) -> Result<Self, ProviderError> {
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

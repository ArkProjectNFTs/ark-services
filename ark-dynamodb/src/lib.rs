//! This crate contains all the common types to work with DynamoDB backend
//! of ark-services.
//!
pub(crate) mod convert;
pub mod metadata_storage;
pub mod pagination;
pub mod providers;
pub mod storage;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
pub use aws_sdk_dynamodb::Client;
use pagination::Lek;
use providers::{
    DynamoDbBlockProvider, DynamoDbContractProvider, DynamoDbEventProvider, DynamoDbTokenProvider,
};
use std::collections::HashMap;
use std::fmt;

/// A context for dynamodb AWS execution.
#[derive(Debug)]
pub struct DynamoDbCtx {
    pub client: Client,
    pub exclusive_start_key: Option<Lek>,
    pub multiple_exclusive_start_keys: HashMap<String, Option<Lek>>,
}

/// A response from dynamodb operation.
#[derive(Debug, Default)]
pub struct DynamoDbOutput<T> {
    inner: T,
    pub lek: Option<HashMap<String, AttributeValue>>,
    pub consumed_capacity_units: Option<f64>,
    pub total_count: Option<i32>,
}

impl<T> DynamoDbOutput<T> {
    pub fn new(inner: T, consumed_capacity_units: Option<f64>, total_count: Option<i32>) -> Self {
        Self {
            inner,
            consumed_capacity_units,
            lek: None,
            total_count,
        }
    }

    pub fn new_lek(
        inner: T,
        consumed_capacity_units: Option<f64>,
        lek: Option<HashMap<String, AttributeValue>>,
        total_count: Option<i32>,
    ) -> Self {
        let mut o = Self::new(inner, consumed_capacity_units, total_count);
        o.lek = lek;
        o
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl From<()> for DynamoDbOutput<()> {
    fn from(unit: ()) -> Self {
        Self {
            inner: unit,
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EntityType {
    Token,
    Block,
    Contract,
    Event,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Token => write!(f, "Token"),
            EntityType::Block => write!(f, "Block"),
            EntityType::Contract => write!(f, "Contract"),
            EntityType::Event => write!(f, "Event"),
        }
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
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Returns a newly initialized DynamoClient.
pub async fn init_aws_dynamo_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    Client::new(&config)
}

/// A convenient provider with all sub-providers.
/// This is not aims to be tested directly, as each provider
/// must be tested separately.
pub struct ArkDynamoDbProvider {
    pub token: DynamoDbTokenProvider,
    pub contract: DynamoDbContractProvider,
    pub event: DynamoDbEventProvider,
    pub block: DynamoDbBlockProvider,
}

impl ArkDynamoDbProvider {
    pub fn new(table_name: &str, limit: Option<i32>) -> Self {
        ArkDynamoDbProvider {
            token: DynamoDbTokenProvider::new(table_name, limit),
            event: DynamoDbEventProvider::new(table_name, limit),
            block: DynamoDbBlockProvider::new(table_name),
            contract: DynamoDbContractProvider::new(table_name, limit),
        }
    }
}

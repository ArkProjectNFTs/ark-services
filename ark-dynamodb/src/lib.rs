//! This crate contains all the common types to work with DynamoDB backend
//! of ark-services.
//!
pub mod metadata_storage;
pub mod pagination;
pub mod providers;
pub mod storage;

pub(crate) mod convert;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::types::{AttributeValue, ConsumedCapacity};
pub use aws_sdk_dynamodb::Client;
use pagination::Lek;
use providers::{
    DynamoDbBlockProvider, DynamoDbContractProvider, DynamoDbEventProvider, DynamoDbTokenProvider,
};
use std::collections::HashMap;
use std::fmt;

pub trait ConsumedCapacityTrait {
    fn total_capacity(&self) -> f64;
}

impl ConsumedCapacityTrait for &Option<ConsumedCapacity> {
    fn total_capacity(&self) -> f64 {
        if let Some(cc) = self {
            cc.capacity_units.unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

impl ConsumedCapacityTrait for &Option<Vec<ConsumedCapacity>> {
    fn total_capacity(&self) -> f64 {
        if let Some(capacities) = self {
            capacities
                .iter()
                .map(|cc| cc.capacity_units.unwrap_or(0.0))
                .sum()
        } else {
            0.0
        }
    }
}

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
    pub capacity: f64,
    pub total_count: Option<i32>,
}

impl<T> DynamoDbOutput<T> {
    pub fn new<C: ConsumedCapacityTrait>(inner: T, consumed_capacity: C) -> Self {
        let capacity = consumed_capacity.total_capacity();

        Self {
            inner,
            capacity,
            lek: None,
            total_count: None,
        }
    }

    pub fn new_lek(
        inner: T,
        consumed_capacity: &Option<ConsumedCapacity>,
        lek: Option<HashMap<String, AttributeValue>>,
        total_count: Option<i32>,
    ) -> Self {
        let mut o = Self::new(inner, consumed_capacity);
        o.lek = lek;
        o.total_count = total_count;
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
}

/// Returns a newly initialized DynamoClient.
pub async fn init_aws_dynamo_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
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

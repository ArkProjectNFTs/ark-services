//! This crate contains all the common types to work with DynamoDB backend
//! of ark-services.
//!
pub mod providers;
pub mod storage;

pub(crate) mod convert;

use aws_config::meta::region::RegionProviderChain;
pub use aws_sdk_dynamodb::Client;
use providers::{
    DynamoDbBlockProvider, DynamoDbContractProvider, DynamoDbEventProvider, DynamoDbTokenProvider,
};
use std::fmt;

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
    #[error("Database error")]
    DatabaseError(String),
    #[error("Missing data error")]
    MissingDataError(String),
    #[error("Data value error")]
    DataValueError(String),
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
    token: DynamoDbTokenProvider,
    contract: DynamoDbContractProvider,
    event: DynamoDbEventProvider,
    block: DynamoDbBlockProvider,
}

impl ArkDynamoDbProvider {
    pub fn new(table_name: &str) -> Self {
        ArkDynamoDbProvider {
            token: DynamoDbTokenProvider::new(table_name),
            event: DynamoDbEventProvider::new(table_name),
            block: DynamoDbBlockProvider::new(table_name),
            contract: DynamoDbContractProvider::new(table_name),
        }
    }
}

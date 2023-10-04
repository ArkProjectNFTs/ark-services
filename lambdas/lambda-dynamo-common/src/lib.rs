//! This crate contains all the common types to work with DynamoDB backend
//! of ark-services.
//!
pub mod collection;
pub(crate) mod convert;
pub mod token;

use aws_config::meta::region::RegionProviderChain;
pub use aws_sdk_dynamodb::Client;

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

/// A default provider type, mostly used for mocking.
#[cfg(any(test, feature = "mock"))]
pub struct MockedClient;

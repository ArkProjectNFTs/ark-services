//! Collection module.
//!
pub mod types;
pub use types::*;

mod dynamo_provider;
pub use dynamo_provider::DynamoDbCollectionProvider;

use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

use crate::ProviderError;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the collection level.
/// Mainly done for mocking purposes, as `Client` from the sdk is not a trait.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkCollectionProvider {
    type Client;

    async fn get_collection(
        &self,
        client: &Self::Client,
        address: &str,
    ) -> Result<Option<CollectionData>, ProviderError>;
}

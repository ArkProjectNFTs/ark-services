//! Block module.
//!
mod dynamo_provider;
pub use dynamo_provider::DynamoDbBlockProvider;

use arkproject::pontos::storage::types::BlockInfo;
use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

use crate::ProviderError;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the block level.
/// Mainly done for mocking purposes, as `Client` from the sdk is not a trait.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkBlockProvider {
    type Client;

    async fn set_info(
        &self,
        client: &Self::Client,
        block_number: u64,
        block_timestamp: u64,
        info: &BlockInfo,
    ) -> Result<(), ProviderError>;

    async fn get_info(
        &self,
        client: &Self::Client,
        block_number: u64,
    ) -> Result<Option<BlockInfo>, ProviderError>;
}

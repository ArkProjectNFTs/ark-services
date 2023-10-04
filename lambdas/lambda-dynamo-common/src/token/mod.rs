//! Token module.
//!
pub mod types;
pub use types::*;

mod dynamo_provider;
pub use dynamo_provider::DynamoDbTokenProvider;

use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

#[cfg(any(test, feature = "mock"))]
use crate::MockedClient;
use crate::ProviderError;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the token level.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=MockedClient;))]
#[async_trait]
pub trait ArkTokenProvider {
    type Client;

    async fn get_token(
        &self,
        client: &Self::Client,
        token_id: &str,
    ) -> Result<TokenData, ProviderError>;
}

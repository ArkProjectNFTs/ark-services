//! Event module.
//!
pub mod types;
pub use types::*;

mod dynamo_provider;
pub use dynamo_provider::DynamoDbEventProvider;

use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

use crate::ProviderError;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the event level.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkEventProvider {
    type Client;

    async fn get_event(
        &self,
        client: &Self::Client,
        contract_address: &str,
        event_id: &str,
    ) -> Result<Option<EventData>, ProviderError>;

    async fn get_token_events(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_id: &str,
    ) -> Result<Vec<EventData>, ProviderError>;
}

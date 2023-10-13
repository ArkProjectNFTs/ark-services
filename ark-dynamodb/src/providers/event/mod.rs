//! Event module.
//!
mod dynamo_provider;
pub use dynamo_provider::DynamoDbEventProvider;

use arkproject::pontos::storage::types::TokenEvent;
use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

use crate::{ProviderError, DynamoDbCtx, DynamoDbOutput};

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the event level.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkEventProvider {
    type Client;

    async fn get_event(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        event_id: &str,
    ) -> Result<DynamoDbOutput<Option<TokenEvent>>, ProviderError>;

    async fn register_event(
        &self,
        ctx: &DynamoDbCtx,
        event: &TokenEvent,
        block_number: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError>;

    async fn get_token_events(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenEvent>>, ProviderError>;

    async fn get_contract_events(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenEvent>>, ProviderError>;
}

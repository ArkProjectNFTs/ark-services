//! Token module.
//!
mod dynamo_provider;
pub use dynamo_provider::DynamoDbTokenProvider;

use arkproject::pontos::storage::types::TokenInfo;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;
use std::collections::HashMap;

use crate::ProviderError;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the token level.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkTokenProvider {
    type Client;

    async fn get_token(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_id: &str,
    ) -> Result<Option<TokenInfo>, ProviderError>;

    async fn register_token(
        &self,
        client: &Self::Client,
        info: &TokenInfo,
        block_number: u64,
    ) -> Result<(), ProviderError>;

    async fn update_data(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_id_hex: &str,
        data: HashMap<String, AttributeValue>,
    ) -> Result<(), ProviderError>;
}

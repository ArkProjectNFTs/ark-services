//! Token module.
//!
mod dynamo_provider;
pub use dynamo_provider::DynamoDbTokenProvider;

use arkproject::pontos::storage::types::TokenInfo;
use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

use crate::ProviderError;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the token level.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkTokenProvider {
    type Client;

    async fn update_owner(
        &self,
        client: &Self::Client,
        contract_address: &str,
        token_id_hex: &str,
        owner: &str,
    ) -> Result<(), ProviderError>;

    async fn update_mint_data(
        &self,
        client: &Self::Client,
        info: &TokenInfo,
    ) -> Result<(), ProviderError>;

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

    async fn get_contract_tokens(
        &self,
        client: &Self::Client,
        contract_address: &str,
    ) -> Result<Vec<TokenInfo>, ProviderError>;
}

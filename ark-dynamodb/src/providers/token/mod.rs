//! Token module.
//!
mod dynamo_provider;
use arkproject::starknet::CairoU256;
pub use dynamo_provider::DynamoDbTokenProvider;
pub mod types;

use arkproject::metadata::types::TokenMetadata;
use arkproject::pontos::storage::types::TokenMintInfo;
use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;
use starknet::core::types::FieldElement;

use crate::providers::token::types::TokenData;
use crate::{DynamoDbCtx, DynamoDbOutput, ProviderError};

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the token level.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkTokenProvider {
    type Client;

    async fn update_owner(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
        owner: &str,
    ) -> Result<DynamoDbOutput<()>, ProviderError>;

    async fn update_mint_info(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
        info: &TokenMintInfo,
    ) -> Result<DynamoDbOutput<()>, ProviderError>;

    async fn update_metadata(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id_hex: &str,
        metadata: &TokenMetadata,
    ) -> Result<DynamoDbOutput<()>, ProviderError>;

    async fn get_token(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        token_id: &str,
    ) -> Result<DynamoDbOutput<Option<TokenData>>, ProviderError>;

    async fn get_token_without_metadata(
        &self,
        client: &Self::Client,
    ) -> Result<Vec<(FieldElement, CairoU256)>, ProviderError>;

    async fn register_token(
        &self,
        ctx: &DynamoDbCtx,
        info: &TokenData,
        block_number: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError>;

    async fn get_contract_tokens(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
        tokens_ids: &[String],
    ) -> Result<DynamoDbOutput<Vec<TokenData>>, ProviderError>;

    async fn get_owner_tokens(
        &self,
        ctx: &DynamoDbCtx,
        owner_address: &str,
    ) -> Result<DynamoDbOutput<Vec<TokenData>>, ProviderError>;
}

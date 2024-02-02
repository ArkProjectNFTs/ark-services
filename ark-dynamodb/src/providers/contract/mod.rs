//! Contract module.
//!
mod dynamo_provider;
use crate::{DynamoDbCtx, DynamoDbOutput, ProviderError};
use arkproject::pontos::storage::types::ContractInfo;
use async_trait::async_trait;
pub use dynamo_provider::DynamoDbContractProvider;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the contract level.
/// Mainly done for mocking purposes, as `Client` from the sdk is not a trait.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkContractProvider {
    type Client;

    async fn get_contract(
        &self,
        ctx: &DynamoDbCtx,
        address: &str,
    ) -> Result<DynamoDbOutput<Option<ContractInfo>>, ProviderError>;

    async fn get_batch_contracts(
        &self,
        ctx: &DynamoDbCtx,
        contract_addresses: Vec<String>,
    ) -> Result<DynamoDbOutput<Vec<ContractInfo>>, ProviderError>;

    async fn register_contract(
        &self,
        ctx: &DynamoDbCtx,
        info: &ContractInfo,
        block_number: u64,
    ) -> Result<DynamoDbOutput<()>, ProviderError>;

    async fn get_nft_contracts(
        &self,
        ctx: &DynamoDbCtx,
    ) -> Result<DynamoDbOutput<Vec<ContractInfo>>, ProviderError>;

    async fn update_nft_contract_image(
        &self,
        ctx: &DynamoDbCtx,
        contract_address: &str,
    ) -> Result<Option<String>, ProviderError>;
}

//! Contract module.
//!
mod dynamo_provider;
pub use dynamo_provider::DynamoDbContractProvider;

use arkproject::pontos::storage::types::ContractInfo;
use async_trait::async_trait;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;

use crate::ProviderError;

/// Trait defining the requests that can be done to dynamoDB for ark-services
/// at the contract level.
/// Mainly done for mocking purposes, as `Client` from the sdk is not a trait.
#[cfg_attr(any(test, feature = "mock"), automock(type Client=aws_sdk_dynamodb::Client;))]
#[async_trait]
pub trait ArkContractProvider {
    type Client;

    async fn get_contract(
        &self,
        client: &Self::Client,
        address: &str,
    ) -> Result<Option<ContractInfo>, ProviderError>;

    async fn register_contract(
        &self,
        client: &Self::Client,
        info: &ContractInfo,
        block_number: u64,
    ) -> Result<(), ProviderError>;

    async fn get_contracts(
        &self,
        client: &Self::Client,
    ) -> Result<Vec<ContractInfo>, ProviderError>;
}

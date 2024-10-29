pub mod block;
pub mod database;
pub mod models;
pub mod types;

use crate::interfaces::contract::{ContractInfo, NFTInfo, TransactionInfo};

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

#[async_trait]
#[cfg_attr(test, automock)]
pub trait Storage {
    async fn store_nft_info(
        &self,
        nft_info: NFTInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn store_transaction_info(
        &self,
        tx_info: TransactionInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn store_token(
        &self,
        nft_info: NFTInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn store_contract(
        &self,
        contract_info: ContractInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn store_token_event(
        &self,
        tx_info: TransactionInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub mod block;
pub mod database;
pub mod models;
pub mod orderbook;
pub mod types;

use crate::interfaces::contract::{ContractInfo, NFTInfo, TransactionInfo};

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use orderbook::OrderbookStorage;

#[async_trait]
#[cfg_attr(test, automock)]
pub trait NFTInfoStorage {
    async fn store_nft_info(
        &self,
        nft_info: NFTInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn store_token(
        &self,
        nft_info: NFTInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait TransactionInfoStorage {
    async fn store_transaction_info(
        &self,
        tx_info: TransactionInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait ContractInfoStorage {
    async fn store_contract(
        &self,
        contract_info: ContractInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub trait Storage:
    NFTInfoStorage + TransactionInfoStorage + ContractInfoStorage + OrderbookStorage
{
}

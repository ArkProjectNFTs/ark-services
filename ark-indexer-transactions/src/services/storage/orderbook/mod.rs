use async_trait::async_trait;

use crate::interfaces::orderbook::OrderTransactionInfo;

pub(crate) mod constants;

pub mod database;

#[async_trait]
pub trait OrderbookStorage {
    async fn store_orderbook_transaction_info(
        &self,
        orderbook_transaction_info: OrderTransactionInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

use ark_dynamodb::storage::AWSDynamoStorage;
use arkproject::pontos::{
    event_handler::EventHandler,
    storage::types::{TokenEvent, TokenInfo},
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct PontosObserver<S: AWSDynamoStorage> {
    _storage: Arc<S>,
    pub indexer_version: String,
    pub indexer_identifier: String,
}

impl<S: AWSDynamoStorage> PontosObserver<S> {
    pub fn new(_storage: Arc<S>, indexer_version: String, indexer_identifier: String) -> Self {
        Self {
            _storage,
            indexer_identifier,
            indexer_version,
        }
    }
}

#[async_trait]
impl<S: AWSDynamoStorage> EventHandler for PontosObserver<S>
where
    S: AWSDynamoStorage + Send + Sync,
{
    async fn on_token_registered(&self, _token: TokenInfo) {}

    async fn on_event_registered(&self, _event: TokenEvent) {}

    async fn on_block_processing(&self, _block_timestamp: u64, _block_numberr: Option<u64>) {}

    async fn on_block_processed(&self, _block_number: u64, _indexation_progresss: f64) {}

    async fn on_indexation_range_completed(&self) {}

    async fn on_new_latest_block(&self, _block_number: u64) {}
}

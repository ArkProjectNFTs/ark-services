use arkproject::pontos::{
    event_handler::EventHandler,
    storage::types::{IndexerStatus, TokenEvent, TokenFromEvent},
};
use async_trait::async_trait;
use log::{info, trace};
use std::sync::Arc;

use crate::dynamo_storage::AWSDynamoStorage;

pub struct PontosObserver<S: AWSDynamoStorage> {
    storage: Arc<S>,
    pub indexer_version: u64,
    pub indexer_identifier: String,
}

impl<S: AWSDynamoStorage> PontosObserver<S> {
    pub fn new(storage: Arc<S>, indexer_version: u64, indexer_identifier: String) -> Self {
        Self {
            storage,
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
    async fn on_token_registered(&self, token: TokenFromEvent) {
        info!("on_token_registered");
    }

    async fn on_event_registered(&self, event: TokenEvent) {
        info!("on_event_registered");
    }

    async fn on_block_processing(&self, block_number: u64) {
        trace!("Block processing: block_number={}", block_number);
        let _ = self
            .storage
            .update_indexer_task_status(
                self.indexer_identifier.clone(),
                self.indexer_version,
                IndexerStatus::Running,
            )
            .await;
    }

    async fn on_terminated(&self, block_number: u64, indexation_progress: f64) {
        trace!("Block processed: block_number={}", block_number);
        let _ = self
            .storage
            .update_indexer_progress(self.indexer_identifier.clone(), indexation_progress)
            .await;
    }
}

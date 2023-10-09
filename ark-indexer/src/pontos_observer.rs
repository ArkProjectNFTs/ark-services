use ark_dynamodb::storage::AWSDynamoStorage;
use arkproject::pontos::{
    event_handler::EventHandler,
    storage::types::{IndexerStatus, TokenEvent, TokenInfo},
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct PontosObserver<S: AWSDynamoStorage> {
    storage: Arc<S>,
    pub indexer_version: String,
    pub indexer_identifier: String,
}

impl<S: AWSDynamoStorage> PontosObserver<S> {
    pub fn new(storage: Arc<S>, indexer_version: String, indexer_identifier: String) -> Self {
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
    async fn on_token_registered(&self, _token: TokenInfo) {
        info!("on_token_registered");
    }

    async fn on_event_registered(&self, _event: TokenEvent) {
        info!("on_event_registered");
    }

    async fn on_block_processing(&self, block_number: u64) {
        debug!(
            "Block processing: block_number={}, indexer_identifier={}, indexer_version={}",
            block_number, self.indexer_identifier, self.indexer_version
        );

        match self
            .storage
            .update_indexer_task_status(
                self.indexer_identifier.clone(),
                self.indexer_version.clone(),
                IndexerStatus::Running,
            )
            .await
        {
            Ok(_) => {
                info!("Task status updated");
            }
            Err(err) => {
                error!("Task status request error: {:?}", err);
            }
        }
    }

    async fn on_terminated(&self, block_number: u64, indexation_progress: f64) {
        info!("Block processed: block_number={}", block_number);
        let _ = self
            .storage
            .update_indexer_progress(self.indexer_identifier.clone(), indexation_progress)
            .await;
    }
}

use ark_dynamodb::storage::AWSDynamoStorage;
use arkproject::pontos::{
    event_handler::EventHandler,
    storage::types::{IndexerStatus, TokenEvent, TokenInfo},
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info};

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

    async fn on_block_processing(&self, block_timestamp: u64, block_number: Option<u64>) {
        info!(
            "Block processing: block_number={:?}, indexer_identifier={}, indexer_version={}, block_timestamp={}",
            block_number, self.indexer_identifier, self.indexer_version, block_timestamp
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

    async fn on_block_processed(&self, block_number: u64, indexation_progress: f64) {
        info!(
            "Block processed: block_number={}, indexation_progress={}",
            block_number, indexation_progress
        );
        let _ = self
            .storage
            .update_indexer_progress(
                self.indexer_identifier.clone(),
                block_number,
                indexation_progress,
            )
            .await;
    }

    async fn on_indexation_range_completed(&self) {
        info!("Indexation completed: {}", self.indexer_identifier);
        let _ = self
            .storage
            .update_indexer_task_status(
                self.indexer_identifier.clone(),
                self.indexer_version.clone(),
                IndexerStatus::Stopped,
            )
            .await;
    }
}

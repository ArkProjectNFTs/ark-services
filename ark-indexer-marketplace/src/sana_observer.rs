use std::sync::Arc;

use arkproject::sana::{
    event_handler::EventHandler,
    storage::{
        types::{TokenEvent, TokenInfo},
        PostgresStorage,
    },
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

pub struct SanaObserver {
    pub storage: Arc<PostgresStorage>,
    pub indexer_identifier: String,
    pub indexer_version: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct BlockRange {
    from_block: u64,
    to_block: u64,
}

impl SanaObserver {
    pub fn new(
        storage: Arc<PostgresStorage>,
        indexer_identifier: String,
        indexer_version: String,
    ) -> Self {
        Self {
            storage,
            indexer_identifier,
            indexer_version,
        }
    }
}

#[async_trait]
impl EventHandler for SanaObserver {
    async fn on_token_registered(&self, _token: TokenInfo) {
        info!("on_token_registered");
    }

    async fn on_event_registered(&self, _event: TokenEvent) {
        info!("on_event_registered");
    }

    async fn on_new_latest_block(&self, block_number: u64) {
        let payload = BlockRange {
            from_block: block_number,
            to_block: block_number,
        };

        match serde_json::to_vec(&payload) {
            Ok(payload_vec) => {
                info!(
                    "New latest block: {} - payload_vec: {:?}",
                    block_number, payload_vec
                );
            }
            Err(err) => error!("Payload serialization error: {:?}", err),
        }

        info!("on_new_latest_block (end)");
    }

    async fn on_block_processed(
        &self,
        block_number: u64,
        indexation_progress: f64,
        force_mode: bool,
        start_block_number: u64,
        end_block_number: u64,
    ) {
        info!(
            "Block processed: block_number={}, indexation_progress={}",
            block_number, indexation_progress
        );

        let _ = self
            .storage
            .update_indexer_progression(
                self.indexer_identifier.as_str(),
                self.indexer_version.as_str(),
                indexation_progress,
                block_number as i64,
                force_mode,
                start_block_number as i64,
                end_block_number as i64,
            )
            .await;
    }
}

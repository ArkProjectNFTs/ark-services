use arkproject::sana::{
    event_handler::EventHandler,
    storage::types::{TokenEvent, TokenInfo},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub struct SanaObserver {
    pub indexer_version: String,
    pub indexer_identifier: String,
}
use tracing::{error, info};

#[derive(Deserialize, Serialize, Debug)]
struct BlockRange {
    from_block: u64,
    to_block: u64,
}

impl SanaObserver {
    pub fn new(indexer_version: String, indexer_identifier: String) -> Self {
        Self {
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
}

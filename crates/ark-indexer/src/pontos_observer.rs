use arkproject::pontos::{
    event_handler::EventHandler,
    storage::types::{TokenEvent, TokenFromEvent},
    storage::Storage,
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct PontosObserver<S> {
    storage: Arc<S>,
}

impl<S> PontosObserver<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<S> EventHandler for PontosObserver<S>
where
    S: Storage + Send + Sync,
{
    /// Pontos has normally terminated the indexation of the given blocks.
    async fn on_terminated(&self, indexer_version: u64, indexer_identifier: &str) {
        // self.storage.set_indexer_progress(progress)
    }

    // /// Block has be processed by Pontos.
    // async fn on_block_processed(
    //     &self,
    //     block_number: u64,
    //     indexer_version: u64,
    //     indexer_identifier: &str,
    // ) {
    //     println!("on_block_processed");
    // }

    // /// A new token has be registered.
    // async fn on_token_registered(&self, token: TokenFromEvent) {
    //     println!("on_token_registered");
    // }

    // /// A new event has be registered.
    // async fn on_event_registered(&self, event: TokenEvent) {
    //     println!("on_event_registered");
    // }

    // TODO: add pertinent events to react on.
}

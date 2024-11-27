use super::manager::ContractManager;
use crate::services::storage::Storage;
use futures::stream::{FuturesUnordered, StreamExt};
use starknet::{
    core::types::Felt,
    providers::{sequencer::models::ConfirmedTransactionReceipt, Provider},
};

use std::error::Error;
use tokio::task::JoinError;

// Définissez un type d'erreur personnalisé qui implémente Send + Sync
#[derive(Debug)]
pub enum EventProcessingError {
    ProcessError(String),
    JoinError(JoinError),
    ThreadError(String),
}

impl std::fmt::Display for EventProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventProcessingError::ProcessError(e) => write!(f, "Processing error: {}", e),
            EventProcessingError::JoinError(e) => write!(f, "Join error: {}", e),
            EventProcessingError::ThreadError(msg) => write!(f, "Thread error: {}", msg),
        }
    }
}

impl Error for EventProcessingError {}

impl From<JoinError> for EventProcessingError {
    fn from(err: JoinError) -> Self {
        EventProcessingError::JoinError(err)
    }
}

impl From<Box<dyn Error + Send + Sync>> for EventProcessingError {
    fn from(err: Box<dyn Error + Send + Sync>) -> Self {
        EventProcessingError::ProcessError(err.to_string())
    }
}

impl<S, P> ContractManager<S, P>
where
    S: Storage + Send + Sync + 'static,
    P: Provider + Send + Sync + 'static,
{
    // pub async fn process_invoke_receipt(
    //     &mut self,
    //     receipt: InvokeTransactionReceipt,
    //     chain_id: Felt,
    //     block_hash: Felt,
    //     tx_hash: Felt,
    //     block_timestamp: u64,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     for event in receipt.events {
    //         self.process_event(event, chain_id, block_hash, tx_hash, block_timestamp)
    //             .await?;
    //     }
    //     Ok(())
    // }

    // pub async fn process_deploy_receipt(
    //     &mut self,
    //     receipt: DeployTransactionReceipt,
    //     chain_id: Felt,
    //     block_hash: Felt,
    //     tx_hash: Felt,
    //     block_timestamp: u64,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     for event in receipt.events {
    //         self.process_event(event, chain_id, block_hash, tx_hash, block_timestamp)
    //             .await?;
    //     }
    //     Ok(())
    // }

    // pub async fn process_deploy_account_receipt(
    //     &mut self,
    //     receipt: DeployAccountTransactionReceipt,
    //     chain_id: Felt,
    //     block_hash: Felt,
    //     tx_hash: Felt,
    //     block_timestamp: u64,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     for event in receipt.events {
    //         self.process_event(event, chain_id, block_hash, tx_hash, block_timestamp)
    //             .await?;
    //     }
    //     Ok(())
    // }

    // pub async fn process_declare_receipt(
    //     &mut self,
    //     receipt: DeclareTransactionReceipt,
    //     chain_id: Felt,
    //     block_hash: Felt,
    //     tx_hash: Felt,
    //     block_timestamp: u64,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     for event in receipt.events {
    //         self.process_event(event, chain_id, block_hash, tx_hash, block_timestamp)
    //             .await?;
    //     }
    //     Ok(())
    // }

    pub async fn common_receipt(
        &mut self,
        receipt: ConfirmedTransactionReceipt,
        chain_id: &str,
        block_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), EventProcessingError> {
        let mut futures = FuturesUnordered::new();
        // println!("start processing event for {:?} on chain  {:?}", block_hash, chain_id);
        for (event_id, event) in receipt.events.into_iter().enumerate() {
            let mut self_clone = self.clone();
            let chain_identifier = chain_id.to_owned();
            futures.push(tokio::spawn(async move {
                self_clone
                    .process_event(
                        event,
                        event_id.try_into().unwrap(),
                        &chain_identifier,
                        block_hash,
                        receipt.transaction_hash,
                        block_timestamp,
                    )
                    .await
            }));
        }

        while let Some(result) = futures.next().await {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e.into()),
                Err(e) => return Err(EventProcessingError::ThreadError(e.to_string())),
            }
        }
        Ok(())
    }

    // pub async fn process_l1_handler_receipt(
    //     &mut self,
    //     receipt: L1HandlerTransactionReceipt,
    //     chain_id: Felt,
    //     block_hash: Felt,
    //     tx_hash: Felt,
    //     block_timestamp: u64,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     for event in receipt.events {
    //         self.process_event(event, chain_id, block_hash, tx_hash, block_timestamp)
    //             .await?;
    //     }
    //     Ok(())
    // }
}

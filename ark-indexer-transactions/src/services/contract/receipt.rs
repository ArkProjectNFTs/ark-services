use crate::services::storage::Storage;
use starknet::{
    core::types::Felt,
    providers::{sequencer::models::ConfirmedTransactionReceipt, Provider},
};

use super::manager::ContractManager;

impl<S: Storage + Send + Sync, P: Provider + Send + Sync> ContractManager<S, P> {
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
        chain_id: Felt,
        block_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // println!("start processing event for {:?} on chain  {:?}", block_hash, chain_id);
        for (event_id, event) in receipt.events.into_iter().enumerate() {
            // println!("start processing event: {:?} on tx {:?}", event, receipt.transaction_hash);
            self.process_event(
                event,
                event_id.try_into().unwrap(),
                chain_id,
                block_hash,
                receipt.transaction_hash,
                block_timestamp,
            )
            .await?;
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

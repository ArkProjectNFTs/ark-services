use crate::helpers::cairo_string_parser::parse_cairo_string;
use crate::interfaces::contract::{ContractType, StarknetClientError, ContractInfo};
use crate::services::state::manager::StateManager;
use std::sync::Arc;
// use crate::services::state::parsing::{load_parsing_state, save_parsing_state, ParsingState};
use crate::services::storage::block::read_block_from_file;
use crate::services::storage::block::BlockWrapper;
use crate::services::storage::Storage;
use num_traits::ToPrimitive;
use starknet::core::types::{BlockId, BlockTag};
use starknet::core::types::{Felt, FunctionCall, StarknetError};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::sequencer::models::ConfirmedTransactionReceipt;
use starknet::providers::sequencer::models::Event;
use starknet::providers::{Provider, ProviderError};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{error, info};
// use super::event::*;
// use super::receipt::*;

const INPUT_TOO_SHORT: &str = "0x496e70757420746f6f2073686f727420666f7220617267756d656e7473";
const INPUT_TOO_LONG: &str = "0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473";
const FAILED_DESERIALIZE: &str = "0x4661696c656420746f20646573657269616c697a6520706172616d202331";
const ENTRYPOINT_NOT_FOUND: &str = "not found in contract";

pub struct ContractManager<S, P>
where
    S: Storage + Send + Sync + 'static,
    P: Provider + Send + Sync + 'static,
{
    pub storage: Arc<Mutex<S>>,
    pub provider: Arc<P>,
    /// A cache with contract address mapped to its type.
    pub cache: Arc<Mutex<HashMap<Felt, ContractType>>>,
    pub orderbooks: Arc<Vec<Felt>>,
}

impl<S, P> Clone for ContractManager<S, P>
where
    S: Storage + Send + Sync + 'static,
    P: Provider + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        ContractManager {
            storage: Arc::clone(&self.storage),
            provider: Arc::clone(&self.provider),
            cache: Arc::clone(&self.cache),
            orderbooks: Arc::clone(&self.orderbooks),
        }
    }
}

impl<S, P> ContractManager<S, P>
where
    S: Storage + Send + Sync + 'static,
    P: Provider + Send + Sync + 'static,
{
    pub fn new(storage: S, provider: P, orderbooks: Vec<Felt>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
            provider: Arc::new(provider),
            cache: Arc::new(Mutex::new(HashMap::new())),
            orderbooks: Arc::new(orderbooks),
        }
    }

    /// Gets the contract info from local cache, or fetch is from the DB.
    pub async fn get_cached_or_fetch_info(
        &mut self,
        address: Felt,
        _chain_id: &str,
    ) -> Result<ContractType, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.lock().await;
        if let Some(contract_type) = cache.get(&address) {
            return Ok(contract_type.clone());
        }
        drop(cache);

        info!("Cache miss for contract {:#064x}", address);

        let contract_type = self.detect_token_standard(address).await?;
        let mut cache = self.cache.lock().await;
        cache.insert(address, contract_type.clone()); // Adding to the cache
        drop(cache);
        Ok(contract_type)
    }

    pub async fn process_block(
        &mut self,
        block: BlockWrapper,
        chain_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "processing block ({:?}) with {:?} txs",
            block.block.block_hash,
            block.block.transaction_receipts.len()
        );

        for tx in block.block.transaction_receipts {
            if let Err(e) = self
                .process_transaction(
                    tx,
                    chain_id,
                    block.block.block_hash.expect("REASON"),
                    block.block.timestamp,
                )
                .await
            {
                error!("Error processing transaction: {:?}", e);
            }
        }
        // for tx in block.block.transaction_receipts {
        //     // println!("Process TXS {:?}", tx.transaction_hash);

        //     self.process_transaction(
        //         tx,
        //         chain_id,
        //         block.block.block_hash.expect("REASON"),
        //         block.block.timestamp,
        //     )
        //     .await?;
        // }
        Ok(())
    }

    pub async fn process_transaction(
        &mut self,
        tx_receipt: ConfirmedTransactionReceipt,
        chain_id: &str,
        block_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.common_receipt(tx_receipt, chain_id, block_hash, block_timestamp)
            .await?;
        // match tx_receipt {
        //     Receipt(tx_receipt) => match tx_receipt {
        //         ConfirmedTransactionReceipt::Invoke(receipt) => {
        //             self.process_invoke_receipt(
        //                 receipt,
        //                 chain_id,
        //                 block_hash,
        //                 receipt.transaction_hash,
        //                 block_timestamp,
        //             )
        //             .await?;
        //         }
        //         TransactionReceipt::Deploy(receipt) => {
        //             self.process_deploy_receipt(
        //                 receipt,
        //                 chain_id,
        //                 block_hash,
        //                 receipt.transaction_hash,
        //                 block_timestamp,
        //             )
        //             .await?;
        //         }
        //         TransactionReceipt::DeployAccount(receipt) => {
        //             self.process_deploy_account_receipt(
        //                 receipt,
        //                 chain_id,
        //                 block_hash,
        //                 receipt.transaction_hash,
        //                 block_timestamp,
        //             )
        //             .await?;
        //         }
        //         TransactionReceipt::Declare(receipt) => {
        //             self.process_declare_receipt(
        //                 receipt,
        //                 chain_id,
        //                 block_hash,
        //                 receipt.transaction_hash,
        //                 block_timestamp,
        //             )
        //             .await?;
        //         }
        //         TransactionReceipt::L1Handler(receipt) => {
        //             self.process_l1_handler_receipt(
        //                 receipt,
        //                 chain_id,
        //                 block_hash,
        //                 receipt.transaction_hash,
        //                 block_timestamp,
        //             )
        //             .await?;
        //         }
        //     },
        //     PendingReceipt(_pending_receipt) => {
        //         todo!()
        //     }
        // }
        Ok(())
    }

    pub async fn identify_contract(
        &mut self,
        address: Felt,
        _block_timestamp: u64,
        chain_id: &str,
    ) -> Result<ContractType, Box<dyn std::error::Error + Send + Sync>> {
        if self.orderbooks.contains(&address) {
            Ok(ContractType::Orderbook)
        } else {
            match self.get_cached_or_fetch_info(address, chain_id).await {
                Ok(contract_type) => Ok(contract_type),
                Err(_) => {
                    if let Ok(contract_type) =
                        self.get_cached_or_fetch_info(address, chain_id).await
                    {
                        return Ok(contract_type);
                    }
                    // If the contract info is not cached, identify and cache it.
                    let contract_type = self.detect_token_standard(address).await?;
                    let mut cache = self.cache.lock().await;
                    cache.insert(address, contract_type.clone());
                    drop(cache);
                    let name = self
                        .get_contract_property_string(
                            address,
                            "name",
                            vec![],
                            BlockId::Tag(BlockTag::Pending),
                        )
                        .await
                        .ok();
                    let symbol = self
                        .get_contract_property_string(
                            address,
                            "symbol",
                            vec![],
                            BlockId::Tag(BlockTag::Pending),
                        )
                        .await
                        .ok();
                    // println!(
                    //     "Contract [0x{:064x}] details - Type: {}, Name: {:?}, Symbol: {:?}",
                    //     address, contract_type, name, symbol
                    // );

                    let _info = ContractInfo {
                        chain_id: chain_id.to_string(),
                        contract_address: address.to_hex_string(),
                        contract_type: contract_type.clone(),
                        name,
                        symbol,
                        image: None,
                    };

                    // if let Err(e) = self
                    //     .storage
                    //     .register_contract_info(&info, block_timestamp)
                    //     .await
                    // {
                    //     error!(
                    //         "Failed to store contract info for [0x{:064x}]: {:?}",
                    //         address, e
                    //     );
                    // }
                    Ok(contract_type)
                }
            }
        }
    }

    /// Verifies if the contract is an ERC721, ERC1155 or an other type.
    /// `owner_of` is specific to ERC721.
    /// `balance_of` is specific to ERC1155 and different from ERC20 as 2 arguments are expected.
    // pub async fn get_contract_type(
    //     &self,
    //     contract_address: Felt,
    // ) -> Result<ContractType, Box<dyn std::error::Error>> {
    //     if self.is_erc721(contract_address).await? {
    //         Ok(ContractType::ERC721)
    //     } else if self.is_erc1155(contract_address).await? {
    //         Ok(ContractType::ERC1155)
    //     } else {
    //         Ok(ContractType::Other)
    //     }
    // }

    pub async fn call_contract(
        &self,
        contract_address: Felt,
        selector: Felt,
        calldata: Vec<Felt>,
        block: BlockId,
    ) -> Result<Vec<Felt>, StarknetClientError> {
        // println!("Call Contract ARGS: Adress: {:?}\n Selector: {:?}\n Call Data: {:?}, block: {:?}", contract_address, selector, calldata, block);
        let provider = self.provider.clone();
        let r = provider
            .call(
                FunctionCall {
                    contract_address,
                    entry_point_selector: selector,
                    calldata,
                },
                block,
            )
            .await;
        drop(provider);
        match r {
            Ok(felts) => Ok(felts),
            Err(e) => {
                if let ProviderError::StarknetError(StarknetError::ContractError(ref data)) = e {
                    let s = data.revert_error.clone();
                    if s.contains(ENTRYPOINT_NOT_FOUND) {
                        Err(StarknetClientError::EntrypointNotFound(s))
                    } else if s.contains(INPUT_TOO_SHORT) || s.contains(FAILED_DESERIALIZE) {
                        Err(StarknetClientError::InputTooShort)
                    } else if s.contains(INPUT_TOO_LONG) {
                        Err(StarknetClientError::InputTooLong)
                    } else {
                        // println!("Eror wile revert Error: {:?}", e);
                        Err(StarknetClientError::Contract(s))
                    }
                } else {
                    // println!("Eror wile provider Call: {:?}", e);
                    Err(StarknetClientError::Contract(e.to_string()))
                }
            }
        }
    }

    pub async fn get_contract_property_string(
        &self,
        contract_address: Felt,
        selector_name: &str,
        calldata: Vec<Felt>,
        block: BlockId,
    ) -> Result<String, StarknetClientError> {
        // println!("Selector Name: {:?}", selector_name);
        let response = self
            .call_contract(
                contract_address,
                get_selector_from_name(selector_name).map_err(|_| {
                    StarknetClientError::Other(format!("Invalid selector: {}", selector_name))
                })?,
                calldata,
                block,
            )
            .await?;
        parse_cairo_string(response).map_err(|e| {
            StarknetClientError::Other(format!("Impossible to decode response string: {:?}", e))
        })
    }

    /// Returns true if the contract is ERC721, false otherwise.
    // pub async fn is_erc721(
    //     &self,
    //     contract_address: Felt,
    // ) -> Result<bool, Box<dyn std::error::Error>> {
    //     let block = BlockId::Tag(BlockTag::Pending);
    //     let token_id = vec![Felt::ONE, Felt::ZERO]; // u256.
    //     match self
    //         .get_contract_response(contract_address, "ownerOf", token_id.clone(), block)
    //         .await
    //     {
    //         Ok(_) => return Ok(true),
    //         Err(e) => match e {
    //             StarknetClientError::Contract(s) => {
    //                 // Token ID may not exist, but the entrypoint was hit.
    //                 if s.contains("not found in contract") {
    //                     // do nothing and go to the next selector.
    //                 } else {
    //                     return Ok(true);
    //                 }
    //             }
    //             StarknetClientError::EntrypointNotFound(_) => (),
    //             _ => return Ok(false),
    //         },
    //     };
    //     match self
    //         .get_contract_response(contract_address, "owner_of", token_id, block)
    //         .await
    //     {
    //         Ok(_) => Ok(true),
    //         Err(e) => match e {
    //             StarknetClientError::Contract(s) => {
    //                 // Token ID may not exist, but the entrypoint was hit.
    //                 if s.contains("not found in contract") {
    //                     Ok(false)
    //                 } else {
    //                     Ok(true)
    //                 }
    //             }
    //             StarknetClientError::EntrypointNotFound(_) => Ok(false),
    //             _ => Ok(false),
    //         },
    //     }
    // }

    /// Returns true if the contract is ERC1155, false otherwise.
    // pub async fn is_erc1155(
    //     &self,
    //     contract_address: Felt,
    // ) -> Result<bool, Box<dyn std::error::Error>> {
    //     let block = BlockId::Tag(BlockTag::Pending);
    //     // felt and u256 expected.
    //     let address_and_token_id = vec![Felt::ZERO, Felt::ONE, Felt::ZERO];

    //     match self
    //         .get_contract_response(
    //             contract_address,
    //             "balanceOf",
    //             address_and_token_id.clone(),
    //             block,
    //         )
    //         .await
    //     {
    //         Ok(_) => return Ok(true),
    //         Err(e) => match e {
    //             StarknetClientError::EntrypointNotFound(_) => (),
    //             StarknetClientError::InputTooLong => return Ok(false), // ERC20.
    //             _ => return Ok(false),
    //         },
    //     };

    //     match self
    //         .get_contract_response(contract_address, "balance_of", address_and_token_id, block)
    //         .await
    //     {
    //         Ok(_) => Ok(true),
    //         Err(e) => match e {
    //             StarknetClientError::EntrypointNotFound(_) => Ok(false),
    //             StarknetClientError::InputTooLong => Ok(false), // ERC20.
    //             _ => Ok(false),
    //         },
    //     }
    // }

    pub async fn get_contract_response(
        &self,
        contract_address: Felt,
        selector_name: &str,
        calldata: Vec<Felt>,
        block: BlockId,
    ) -> Result<Vec<Felt>, StarknetClientError> {
        self.call_contract(
            contract_address,
            get_selector_from_name(selector_name).map_err(|_| {
                StarknetClientError::Other(format!("Invalid selector: {}", selector_name))
            })?,
            calldata,
            block,
        )
        .await
    }

    pub async fn process_event(
        &mut self,
        event: Event,
        event_id: u64,
        chain_id: &str,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // println!("Processing {}", event_id);
        let contract_address = event.from_address;
        let contract_type = self
            .identify_contract(contract_address, block_timestamp, chain_id)
            .await?;
        // println!("contract-Type: {:?} for {:?}", contract_type, contract_address);
        // println!("contract_type {}", contract_type);
        match contract_type {
            ContractType::ERC20 => {
                // ERC20 handling is currently disabled
                let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = Ok(());
                result?
            }
            ContractType::ERC721 => {
                self.handle_erc721_event(
                    event,
                    event_id,
                    chain_id,
                    block_hash,
                    tx_hash,
                    block_timestamp,
                )
                .await?
            }
            ContractType::ERC1155 => {
                self.handle_erc1155_event(
                    event,
                    event_id,
                    chain_id,
                    block_hash,
                    tx_hash,
                    block_timestamp,
                )
                .await?
            }
            ContractType::ERC1400 => {
                // ERC1400 handling is currently disabled
                let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = Ok(());
                result?
            }
            ContractType::Orderbook => {
                self.handle_orderbook_event(
                    event,
                    event_id,
                    chain_id,
                    block_hash,
                    tx_hash,
                    block_timestamp,
                )
                .await?
            }
            ContractType::Other => {
                self.handle_other_event(
                    event,
                    event_id,
                    chain_id,
                    block_hash,
                    tx_hash,
                    block_timestamp,
                )
                .await?
            }
        }

        Ok(())
    }

    pub async fn index_blocks(
        &mut self,
        from_block: u64,
        to_block: u64,
        base_path: &str,
        parsing_state_path: &str,
        chain_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state_manager = StateManager::new(parsing_state_path).unwrap();

        // match parsing_state.get_block_state(block_number) {
        //     Ok(state) => match verify_block_format(&block_path) {
        //         Ok(_block_number_verified) => {
        //             if !state {
        //                 state_manager
        //                     .set_block_state(block_number, true)
        //                     .unwrap();
        //             }
        //         }
        //         Err(e) => {
        //             state_manager
        //                 .set_block_state(block_number, false)
        //                 .unwrap();
        //             eprintln!(
        //                 "Failed to verify block format for {:?}: {}",
        //                 block_path, e
        //             );
        //         }
        //     },
        //     Err(e) => {
        //         state_manager.set_block_state(block_number, false).unwrap();
        //         eprintln!(
        //             "Failed to verify state for {:?}: {}",
        //             block_path, e
        //         );
        //     }
        // };

        // let mut parsing_state = match load_parsing_state(parsing_state_path) {
        //     Ok(state) => state,
        //     Err(_) => ParsingState::new(),
        // };

        let start_block = match state_manager.get_last_processed_block() {
            Ok(last_block) => {
                let last_block_u64 = last_block.to_u64().unwrap();
                if from_block <= last_block_u64 {
                    last_block_u64
                } else {
                    from_block
                }
            }
            Err(_e) => from_block,
        };

        for block_number in start_block..=to_block {
            if state_manager
                .get_block_state(block_number.to_usize().unwrap())
                .unwrap_or(false)
            {
                continue;
            }

            match read_block_from_file(base_path, block_number) {
                Ok(block) => {
                    self.process_block(block, chain_id).await?;
                    let _ = state_manager.set_block_state(block_number.try_into().unwrap(), true);
                }
                Err(e) => {
                    error!("Error reading block {}: {:?}", block_number, e);
                }
            }
        }

        Ok(())
    }

    // pub async fn reindex_pending_block(&mut self, parsing_state_path: &str, chain_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     let pending_block_number = 0;
    //     if let Ok(block) = read_block_from_file(pending_block_number) {
    //         let block_hash = block.block.block_hash.to_hex_string();

    //         let mut parsing_state = match load_parsing_state(parsing_state_path) {
    //             Ok(state) => state,
    //             Err(_) => ParsingState::new(),
    //         };

    //         if !parsing_state.is_block_indexed(pending_block_number) {
    //             self.reindex_pending_block_logic(block, chain_id, &block_hash).await?;
    //             parsing_state.mark_block_indexed(pending_block_number);
    //             save_parsing_state(&parsing_state, parsing_state_path)?;
    //         }
    //     }

    //     Ok(())
    // }

    // async fn reindex_pending_block_logic(&self, pending_block: BlockWrapper, chain_id: &str, block_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     for tx in pending_block.block.transactions {
    //         if let Some(receipt) = tx.receipt {
    //             for event in receipt.events {
    //                 let event_id = self.get_event_id(&event);

    //                 if self.storage.is_event_already_indexed(&event_id, block_hash).await? {
    //                     continue;
    //                 }

    //                 self.process_event(event, chain_id).await?;
    //                 // self.process_event(event, chain_id, block_hash).await?;
    //             }
    //         }
    //     }
    //     Ok(())
    // }
}

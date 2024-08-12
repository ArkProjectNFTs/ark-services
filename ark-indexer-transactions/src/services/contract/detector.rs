use crate::{
    interfaces::{contract::ContractType, event::ErcAction},
    services::storage::Storage,
};

use super::manager::ContractManager;
use starknet::{
    core::types::{
        BlockId::{self},
        BlockTag,
        ContractClass::{self, Legacy, Sierra},
        Felt,
    },
    providers::Provider,
};

use starknet::core::types::LegacyContractAbiEntry::Function;

impl<S: Storage + Send + Sync, P: Provider + Send + Sync> ContractManager<S, P> {
    pub fn detect_erc_action(&self, from: Felt, to: Felt) -> ErcAction {
        if from == Felt::ZERO {
            ErcAction::MINT
        } else if to == Felt::ZERO {
            ErcAction::BURN
        } else {
            ErcAction::OTHER
        }
    }

    pub async fn detect_token_standard(
        &self,
        contract_address: Felt,
    ) -> Result<ContractType, Box<dyn std::error::Error>> {
        // Fetch the contract class using the provider
        // println!("contract_address: {:?}", contract_address);
        let provider = self.provider.lock().await;
        let class: ContractClass = provider
            .get_class_at(BlockId::Tag(BlockTag::Pending), contract_address)
            .await?;
        drop(provider);

        // Determine the ERC standard by inspecting the methods
        if self.is_erc20(&class) {
            // println!("ERC20 Detected");
            return Ok(ContractType::ERC20);
        } else if self.is_erc721(&class) {
            // println!("ERC721 Detected");
            return Ok(ContractType::ERC721);
        } else if self.is_erc1155(&class) {
            // println!("ERC1155 Detected");
            return Ok(ContractType::ERC1155);
        } else if self.is_erc1400(&class) {
            // println!("ERC1400 Detected");
            return Ok(ContractType::ERC1400);
        }
        Ok(ContractType::Other)
    }

    fn is_erc20(&self, class: &ContractClass) -> bool {
        // Check if the class has the typical ERC20 functions
        (self.has_function(class, "balanceOf") || self.has_function(class, "balance_of"))
        // && self.has_function(class, "transfer")
        && (self.has_function(class, "total_supply") || self.has_function(class, "totalSupply"))
        && self.has_function(class, "allowance")
        // && self.has_function(class, "approve")
        // && self.has_function(class, "transferFrom")
    }

    fn is_erc721(&self, class: &ContractClass) -> bool {
        // Check if the class has the typical ERC721 functions
        // (self.has_function(class, "balanceOf") || self.has_function(class, "balance_of"))
        (self.has_function(class, "ownerOf") || self.has_function(class, "owner_of"))
            && (self.has_function(class, "tokenURI")
                || self.has_function(class, "token_uri")
                || self.has_function(class, "uri"))
    }

    fn is_erc1155(&self, class: &ContractClass) -> bool {
        // Check if the class has the typical ERC1155 functions
        (self.has_function(class, "balanceOf") || self.has_function(class, "balance_of"))
        // && self.has_function(class, "balanceOfBatch")
        && self.has_function(class, "safeTransferFrom")
        // && self.has_function(class, "safeBatchTransferFrom")
    }

    fn is_erc1400(&self, class: &ContractClass) -> bool {
        // Check if the class has the typical ERC1400 functions
        (self.has_function(class, "balanceOf") || self.has_function(class, "balance_of"))
            && self.has_function(class, "transferWithData")
        // && self.has_function(class, "transferWithData")
        // && self.has_function(class, "redeem")
        // && self.has_function(class, "issue")
    }

    fn has_function(&self, class: &ContractClass, function_name: &str) -> bool {
        // Check if a function with the given name exists in the class methods
        let ret = match class {
            Sierra(sierra_class) => {
                // println!("EntryPoints: {:?}", sierra_class.entry_points_by_type.constructor);
                sierra_class
                    .entry_points_by_type
                    .constructor
                    .iter()
                    .any(|entry| {
                        // let a = entry.selector;
                        // let b = starknet::core::utils::get_selector_from_name(function_name).unwrap();
                        entry.selector
                            == starknet::core::utils::get_selector_from_name(function_name).unwrap()
                        // println!("A : {:?} and B : {:?},{:?}", a, b, function_name);
                        // a == b
                    })
            }
            Legacy(legacy_class) => {
                match &legacy_class.abi {
                    Some(entries) => {
                        entries.iter().any(|entry| {
                            match entry {
                                Function(function_call) => {
                                    // println!("Function call name {} compared to {}", function_call.name, function_name);
                                    function_call.name == *function_name
                                }
                                _ => false,
                            }
                        })
                    }
                    None => false,
                }
                // println!("EntryPoints: {:?}", legacy_class.entry_points_by_type.constructor);
                // legacy_class.entry_points_by_type.constructor.iter()
                // .any(|entry| {
                //     let a = entry.selector;
                //     let b = starknet::core::utils::get_selector_from_name(function_name).unwrap();
                //     println!("A : {:?} and B : {:?},{:?}", a, b, function_name);
                //     a == b
                // })
            }
        };
        // println!("RET {}", ret);
        ret
    }
}

#[test]
fn should_decode_the_same_way() {
    let select = starknet::core::utils::get_selector_from_name("balanceOf").unwrap();
    let selector =
        Felt::from_hex("0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e")
            .unwrap();
    assert_eq!(select, selector);
}

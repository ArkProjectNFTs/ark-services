use crate::{interfaces::contract::ContractType, services::storage::Storage};

use super::{erc1155, erc1400, erc20, erc721, manager::ContractManager};
use starknet::{
    core::types::{
        BlockId::{self},
        BlockTag,
        ContractClass::{self},
        Felt,
    },
    providers::Provider,
};

impl<S, P> ContractManager<S, P>
where
    S: Storage + Send + Sync + 'static,
    P: Provider + Send + Sync + 'static,
{
    pub async fn detect_token_standard(
        &self,
        contract_address: Felt,
    ) -> Result<ContractType, Box<dyn std::error::Error + Send + Sync>> {
        // Fetch the contract class using the provider
        // println!("contract_address: {:?}", contract_address);
        let provider = self.provider.clone();
        let class: ContractClass = provider
            .get_class_at(BlockId::Tag(BlockTag::Pending), contract_address)
            .await?;
        drop(provider);

        // Determine the ERC standard by inspecting the methods
        if erc20::detect(&class) {
            // println!("ERC20 Detected");
            return Ok(ContractType::ERC20);
        } else if erc1155::detect(&class) {
            // println!("ERC1155 Detected");
            return Ok(ContractType::ERC1155);
        } else if erc721::detect(&class) {
            // println!("ERC721 Detected");
            return Ok(ContractType::ERC721);
        } else if erc1400::detect(&class) {
            // println!("ERC1400 Detected");
            return Ok(ContractType::ERC1400);
        }
        Ok(ContractType::Other)
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

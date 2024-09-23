use crate::services::contract::common::has_function;
use starknet::core::types::ContractClass;

pub fn detect(class: &ContractClass) -> bool {
    // Check if the class has the typical ERC721 functions
    // (has_function(class, "balanceOf") || has_function(class, "balance_of"))
    (has_function(class, "ownerOf") || has_function(class, "owner_of"))
        && (has_function(class, "tokenURI")
            || has_function(class, "token_uri")
            || has_function(class, "uri"))
}

use crate::services::contract::common::has_function;
use starknet::core::types::ContractClass;

pub fn detect(class: &ContractClass) -> bool {
    // Check if the class has the typical ERC20 functions
    (has_function(class, "balanceOf") || has_function(class, "balance_of"))
        // && has_function(class, "transfer")
        && (has_function(class, "total_supply") || has_function(class, "totalSupply"))
        && has_function(class, "allowance")
    // && has_function(class, "approve")
    // && has_function(class, "transferFrom")
}

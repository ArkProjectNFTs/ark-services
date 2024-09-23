use crate::services::contract::common::has_function;
use starknet::core::types::ContractClass;

pub fn detect(class: &ContractClass) -> bool {
    // Check if the class has the typical ERC1400 functions
    (has_function(class, "balanceOf") || has_function(class, "balance_of"))
        && has_function(class, "transferWithData")
    // && has_function(class, "transferWithData")
    // && has_function(class, "redeem")
    // && has_function(class, "issue")
}

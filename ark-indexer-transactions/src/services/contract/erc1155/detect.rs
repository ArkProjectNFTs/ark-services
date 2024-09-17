use crate::services::contract::common::has_function;
use starknet::core::types::ContractClass;

pub fn detect(class: &ContractClass) -> bool {
    // Check if the class has the typical ERC1155 functions
    (has_function(class, "balanceOf") || has_function(class, "balance_of"))
        // && has_function(class, "balanceOfBatch")
        && (has_function(class, "safeBatchTransferFrom")
            || has_function(class, "safe_batch_transfer_from"))
    // && has_function(class, "safeBatchTransferFrom")
}

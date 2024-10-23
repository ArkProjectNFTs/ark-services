pub mod utils;
use crate::interfaces::event::ErcAction;
use starknet::core::types::{ContractClass, Felt};

pub fn detect_erc_action(from: Felt, to: Felt) -> ErcAction {
    if from == Felt::ZERO {
        ErcAction::MINT
    } else if to == Felt::ZERO {
        ErcAction::BURN
    } else {
        ErcAction::OTHER
    }
}

pub fn has_function(class: &ContractClass, function_name: &str) -> bool {
    match class {
        ContractClass::Sierra(sierra_class) => {
            sierra_class.entry_points_by_type.external.iter().any(|entry| {
                entry.selector == starknet::core::utils::get_selector_from_name(function_name).unwrap()
            })
        }
        ContractClass::Legacy(legacy_class) => {
            legacy_class.abi.as_ref().map_or(false, |entries| {
                entries.iter().any(|entry| {
                    matches!(entry, starknet::core::types::LegacyContractAbiEntry::Function(f) if f.name == function_name)
                })
            })
        }
    }
}

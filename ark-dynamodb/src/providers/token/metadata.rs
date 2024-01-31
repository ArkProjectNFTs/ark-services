use starknet::core::types::FieldElement;

pub fn create_contract_filter(contract_address_filter: Option<FieldElement>) -> String {
    match contract_address_filter {
        Some(contract_address) => format!("CONTRACT#0x{:064x}", contract_address),
        None => "CONTRACT#".to_string(),
    }
}

pub fn get_excluded_contracts() -> Vec<String> {
    vec![
        "0x07b696af58c967c1b14c9dde0ace001720635a660a8e90c565ea459345318b30".to_string(),
        "0x07f5e93a406f46c49ad2fdeb013a5f25ef5d2dc5a658b6832d6c421898399aa4".to_string(),
        "0x00fff107e2403123c7df78d91728a7ee5cfd557aec0fa2d2bdc5891c286bbfff".to_string(),
        "0x01bd387d18e52e0a04a87c5f9232e9b3cbd1d630837926e6fece2dea4a65bea9".to_string(),
        "0x04a3621276a83251b557a8140e915599ae8e7b6207b067ea701635c0d509801e".to_string(),
        "0x03f96949d14c65ec10e7544d93f298d0cb07c498ecb733774f1d4b2adf3ffb23".to_string(),
    ]
}

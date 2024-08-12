use starknet_crypto::Felt;

pub fn felt_to_strk_string(value: Felt) -> String {
    if value == Felt::ZERO {
        value.to_hex_string()
    } else {
        value.to_fixed_hex_string()
    }
}

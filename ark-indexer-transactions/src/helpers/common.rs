use starknet_crypto::Felt;

pub fn felt_to_strk_string(value: Felt) -> String {
    if value == Felt::ZERO {
        value.to_hex_string()
    } else {
        value.to_fixed_hex_string()
    }
}

pub fn sanitize_string(input: &str) -> String {
    // Remove null bytes and ensure valid UTF-8
    input
        .chars()
        .filter(|&c| c != '\0' && c.is_ascii())
        .collect::<String>()
}

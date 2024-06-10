use num_bigint::BigUint;
use std::str::FromStr;

/// Returns the padded hex string of parameter that can be an hexadecimal / decimal string.
/// Returns an error if the parameter is not found.
#[allow(clippy::bind_instead_of_map)]
pub fn convert_param_to_hex(param: &str) -> Result<String, &'static str> {
    if param.starts_with("0x") || param.starts_with("0X") {
        Ok(param.to_string())
    } else {
        match BigUint::from_str(param) {
            Ok(num) => Ok(format!("0x{:064x}", num)),
            Err(_) => Err("Convert param to hex error"),
        }
    }
}

/// Normalize an Ethereum address to a consistent length of 66 characters (including the '0x' prefix)
/// by padding with leading zeros. This is useful for ensuring that Ethereum addresses are always
/// represented in the same way, regardless of how they were input or received.
///
/// # Arguments
///
/// * `address` - An address as a string. It should start with '0x'.
///
/// # Returns
///
/// * A string representing the normalized Ethereum address.
pub fn normalize_address(address: &str) -> String {
    let prefix = &address[0..2];
    let hex = &address[2..];

    let hex = format!("{:0>64}", hex);

    let normalized_address = format!("{}{}", prefix, hex);

    normalized_address
}

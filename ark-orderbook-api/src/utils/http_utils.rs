use num_bigint::BigUint;
use std::str::FromStr;

/// Returns the padded hex string of parameter that can be an hexadecimal / decimal string.
/// Returns an error if the parameter is not found.
#[allow(clippy::bind_instead_of_map)]
pub fn convert_param_to_hex(
    param: &str,
) -> Result<String, &'static str> {
    if param.starts_with("0x") || param.starts_with("0X") {
        Ok(param.to_string())
    } else {
        match BigUint::from_str(param) {
            Ok(num) => Ok(format!("0x{:064x}", num)),
            Err(_) => Err("Convert param to hex error"),
        }
    }
}

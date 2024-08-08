use num_bigint::BigUint;
use reqwest;
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

/**
 * Get the address from the Starknet ID.
 *
 * # Arguments
 *
 * * `starknet_id` - The Starknet ID.
 *
 * # Returns
 *
 * * The address if it exists.
 */
pub async fn get_address_from_starknet_id(
    starknet_id: &str,
) -> Result<Option<String>, reqwest::Error> {
    let url = format!(
        "https://api.starknet.id/domain_to_addr?domain={}",
        starknet_id
    );
    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        if let Some(address) = json["addr"].as_str() {
            return Ok(Some(address.to_string()));
        }
    }
    Ok(None)
}

/**
 * Get the Starknet ID from the address.
 *
 * # Arguments
 *
 * * `address` - address.
 *
 * # Returns
 *
 * * The Starknet ID if it exists.
 */
pub async fn get_starknet_id_from_address(address: &str) -> Result<Option<String>, reqwest::Error> {
    let url = format!("https://api.Starknet.id/addr_to_domain?addr={}", address);
    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        if let Some(address) = json["domain"].as_str() {
            return Ok(Some(address.to_string()));
        }
    }
    Ok(None)
}

/**
 * The API will return all the Starknet IDs owned by the address.
 *
 * # Arguments
 *
 * * `address` - address.
 *
 * # Returns
 *
 */
pub async fn get_image_from_starknet_address(
    address: &str,
) -> Result<Option<String>, reqwest::Error> {
    let url = format!("https://api.Starknet.id/addr_to_full_ids?addr={}", address);
    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        if let Some(full_ids) = json["full_ids"].as_array() {
            if let Some(first_id) = full_ids.get(0) {
                if let Some(pp_url) = first_id["pp_url"].as_str() {
                    return Ok(Some(pp_url.to_string()));
                }
            }
        }
    }
    Ok(None)
}

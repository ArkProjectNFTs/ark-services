use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use crate::ProviderError;

/// Returns a HashMap<String, AttributeValue> from the given attribute, `ProviderError` if data is missing or invalid data value.
pub fn attr_to_map(
    data: &HashMap<String, AttributeValue>,
    attr: &str,
) -> Result<HashMap<String, AttributeValue>, ProviderError> {
    if let Some(a) = data.get(attr) {
        Ok(a.as_m()
            .map_err(|_e| {
                ProviderError::DataValueError(format!("Expecting M for attribute {}", attr))
            })?
            .clone())
    } else {
        Err(ProviderError::MissingDataError(format!(
            "No data found for attr {}",
            attr
        )))
    }
}

/// Returns the `u64` value for the given attribute, `ProviderError` if data is missing or invalid data value.
pub fn attr_to_u64(
    data: &HashMap<String, AttributeValue>,
    attr: &str,
) -> Result<u64, ProviderError> {
    if let Some(a) = data.get(attr) {
        let n = a.as_n().map_err(|_e| {
            ProviderError::DataValueError(format!("Expecting N for attribute {}", attr))
        })?;

        Ok(n.parse::<u64>().map_err(|_e| {
            ProviderError::DataValueError(format!("Expecting u64 for attribute {}", attr))
        })?)
    } else {
        Err(ProviderError::MissingDataError(format!(
            "No data found for attr {}",
            attr
        )))
    }
}

/// Returns the `String` value for the given attribute, `ProviderError` if data is missing or invalid data value.
pub fn attr_to_str(
    data: &HashMap<String, AttributeValue>,
    attr: &str,
) -> Result<String, ProviderError> {
    if let Some(a) = data.get(attr) {
        let s = a.as_s().map_err(|_e| {
            ProviderError::DataValueError(format!("Expecting S for attribute {}", attr))
        })?;

        Ok(s.to_string())
    } else {
        Err(ProviderError::MissingDataError(format!(
            "No data found for attr {}",
            attr
        )))
    }
}

/// Returns the `String` value for the given attribute, or None if not found.
pub fn attr_to_opt_str(
    data: &HashMap<String, AttributeValue>,
    attr: &str,
) -> Result<Option<String>, ProviderError> {
    if let Some(a) = data.get(attr) {
        let s = a.as_s().map_err(|_e| {
            ProviderError::DataValueError(format!("Expecting S for attribute {}", attr))
        })?;

        Ok(Some(s.to_string()))
    } else {
        Ok(None)
    }
}

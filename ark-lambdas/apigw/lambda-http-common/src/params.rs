use lambda_http::{Request, RequestExt};

use crate::format::*;
use crate::LambdaHttpError;

/// Source of the parameter in the HTTP request.
pub enum HttpParamSource {
    Path,
    QueryString,
}

/// Returns the value for the given parameter as a string. Returns an error if the parameter is not found.
#[allow(clippy::bind_instead_of_map)]
pub fn string_param(event: &Request, param_name: &str, source: HttpParamSource) -> Option<String> {
    match source {
        HttpParamSource::Path => event
            .path_parameters_ref()
            .and_then(|params| Some(params.first(param_name)))
            .unwrap_or(None)
            .map(|v| v.to_string()),
        HttpParamSource::QueryString => event
            .query_string_parameters_ref()
            .and_then(|params| Some(params.first(param_name)))
            .unwrap_or(None)
            .map(|v| v.to_string()),
    }
}

/// Returns the value for the given parameter as a string. Returns an error if the parameter is not found.
#[allow(clippy::bind_instead_of_map)]
pub fn require_string_param(
    event: &Request,
    param_name: &str,
    source: HttpParamSource,
) -> Result<String, LambdaHttpError> {
    let maybe_param = string_param(event, param_name, source);

    if let Some(v) = maybe_param {
        Ok(v)
    } else {
        Err(LambdaHttpError::ParamMissing(format!(
            "Param {param_name} is missing"
        )))
    }
}

/// Returns the padded hex string of parameter that can be an hexadecimal / decimal string.
/// Returns an error if the parameter is not found.
#[allow(clippy::bind_instead_of_map)]
pub fn require_hex_or_dec_param(
    event: &Request,
    param_name: &str,
    source: HttpParamSource,
) -> Result<String, LambdaHttpError> {
    let maybe_param = string_param(event, param_name, source);

    if let Some(v) = maybe_param {
        Ok(hex_or_dec_from_str(&v, param_name)?)
    } else {
        Err(LambdaHttpError::ParamMissing(format!(
            "Param {param_name} is missing"
        )))
    }
}

/// Returns the value of a parameter expected to be an hexadecimal string.
/// Returns an error if the parameter is not found.
#[allow(clippy::bind_instead_of_map)]
pub fn require_hex_param(
    event: &Request,
    param_name: &str,
    source: HttpParamSource,
) -> Result<String, LambdaHttpError> {
    let maybe_param = string_param(event, param_name, source);

    if let Some(v) = maybe_param {
        if is_hexadecimal_with_prefix(&v) {
            Ok(pad_hex(&v))
        } else {
            Err(LambdaHttpError::ParamParsing(format!(
                "Param {param_name} is expected to be hexadecimal string"
            )))
        }
    } else {
        Err(LambdaHttpError::ParamMissing(format!(
            "Param {param_name} is missing"
        )))
    }
}

use lambda_http::{Body, Error, Request, RequestExt, Response};
use num_bigint::BigUint;
use num_traits::Num;
use serde::Serialize;

/// Generic errors for http parsing.
#[derive(Debug, thiserror::Error)]
pub enum HttpParsingError {
    #[error("Bad param")]
    ParamError(String),
    #[error("Missing param")]
    MissingParamError(String),
}

impl TryFrom<HttpParsingError> for Response<Body> {
    type Error = Error;

    fn try_from(e: HttpParsingError) -> Result<Self, Self::Error> {
        Ok(Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body(match e {
                HttpParsingError::ParamError(s) => s.into(),
                HttpParsingError::MissingParamError(s) => s.into(),
            })
            .map_err(Box::new)?)
    }
}

/// Returns a `Response` with OK status and the given body serialized as a JSON.
pub fn ok_body_rsp<T: Serialize>(body: &T) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&body)?.into())
        .map_err(Box::new)?)
}

/// Returns a `Response` with NOT_FOUND status.
pub fn not_found_rsp() -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "text/plain")
        .body("".into())
        .map_err(Box::new)?)
}

/// Returns a `Reponse` with BAD_REQUEST status and the given message as body.
pub fn bad_request_rsp(message: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(400)
        .header("Content-Type", "text/plain")
        .body(message.into())
        .map_err(Box::new)?)
}

/// Returns the value for the given param name into the query string parameters, None otherwise.
#[allow(clippy::bind_instead_of_map)]
pub fn get_query_string_param(
    event: &Request,
    param_name: &str,
) -> Result<String, HttpParsingError> {
    if let Some(v) = event
        .query_string_parameters_ref()
        .and_then(|params| Some(params.first(param_name)))
        .unwrap_or(None)
        .map(|v| v.to_string())
    {
        Ok(v)
    } else {
        Err(HttpParsingError::MissingParamError(format!(
            "Param {param_name} is missing"
        )))
    }
}

/// Returns the padded hex string of a query string param expected to be a hex or
/// a decimal number.
#[allow(clippy::bind_instead_of_map)]
pub fn get_query_string_hex_or_dec_param(
    event: &Request,
    param_name: &str,
) -> Result<String, HttpParsingError> {
    if let Some(v) = event
        .query_string_parameters_ref()
        .and_then(|params| Some(params.first(param_name)))
        .unwrap_or(None)
        .map(|v| v.to_string())
    {
        if v.starts_with("0x") {
            if is_hexadecimal_with_prefix(&v) {
                Ok(pad_hex(&v))
            } else {
                Err(HttpParsingError::ParamError(format!(
                    "Param {param_name} is expected to be valid hex string or decimal string"
                )))
            }
        } else {
            let biguint = match BigUint::from_str_radix(&v, 10) {
                Ok(i) => i,
                Err(_) => {
                    return Err(HttpParsingError::ParamError(format!(
                        "Param {param_name} out of range decimal value"
                    )))
                }
            };

            // We always work with fully padded hex strings.
            Ok(format!("0x{:064x}", biguint))
        }
    } else {
        Err(HttpParsingError::MissingParamError(format!(
            "Param {param_name} is missing"
        )))
    }
}

/// Returns the value of a query string param expected to be a hexadecimal string.
#[allow(clippy::bind_instead_of_map)]
pub fn get_query_string_hex_param(
    event: &Request,
    param_name: &str,
) -> Result<String, HttpParsingError> {
    if let Some(v) = event
        .query_string_parameters_ref()
        .and_then(|params| Some(params.first(param_name)))
        .unwrap_or(None)
        .map(|v| v.to_string())
    {
        if is_hexadecimal_with_prefix(&v) {
            Ok(pad_hex(&v))
        } else {
            Err(HttpParsingError::ParamError(format!(
                "Param {param_name} is expected to be hexadecimal string"
            )))
        }
    } else {
        Err(HttpParsingError::MissingParamError(format!(
            "Param {param_name} is missing"
        )))
    }
}

/// Pads an hexadecimal value to be 32 bytes long + 0x prefix.
pub fn pad_hex(input: &str) -> String {
    if input.len() > 64 + 2 {
        return String::new();
    }

    if input.len() == 64 + 2 {
        return input.to_string();
    }

    let s = input.strip_prefix("0x").unwrap_or(input);

    let mut padded = String::with_capacity(64);
    let padding_count = 64 - s.len();

    for _ in 0..padding_count {
        padded.push('0');
    }

    padded.push_str(s);

    format!("0x{padded}")
}

/// Returns true if the given string is an hexadecimal string with `0x` prefix, false otherwise.
pub fn is_hexadecimal_with_prefix(input: &str) -> bool {
    if input.len() < 3 {
        return false;
    }

    if &input[0..2] != "0x" {
        return false;
    }

    for c in input[2..].chars() {
        if !c.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_is_hexadecimal_with_prefix() {
        let s = "0x123";
        assert!(is_hexadecimal_with_prefix(s));
    }

    #[test]
    fn test_is_hexadecimal_with_prefix_invalid() {
        let s = "0x";
        assert!(is_hexadecimal_with_prefix(s));

        let s = "1234";
        assert!(is_hexadecimal_with_prefix(s));
    }

    #[test]
    fn test_pad_hex() {
        assert_eq!(pad_hex("0x12"), format!("0x{:064x}", 0x12));
        assert_eq!(pad_hex("12"), format!("0x{:064x}", 0x12));
    }

    #[test]
    fn test_get_query_string_param() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "0x1234".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_param(&req, "address").unwrap();
        assert_eq!(s, "0x1234");

        let s = get_query_string_param(&req, "other_param");
        assert!(s.is_err());
    }

    #[test]
    fn test_get_query_string_param_hex() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "0x1234".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_hex_param(&req, "address").unwrap();
        assert_eq!(s, format!("0x{:064x}", 0x1234));
    }

    #[test]
    fn test_get_query_string_param_hex_invalid() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "1234".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_hex_param(&req, "address");
        assert!(s.is_err());

        let mut params = HashMap::new();
        params.insert("address".to_string(), "jfeoehguoirehgo".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_hex_param(&req, "address");
        assert!(s.is_err());
    }

    #[test]
    fn test_get_query_string_param_hex_or_dec() {
        let mut params = HashMap::new();
        params.insert("token_id".to_string(), "255".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_hex_or_dec_param(&req, "token_id").unwrap();
        assert_eq!(s, format!("0x{:064x}", 0xff));

        let mut params = HashMap::new();
        params.insert(
            "token_id".to_string(),
            "0x6f5b84a20c71f393a8b5b4d74f62df2017b80bb5dcef199e2fa6952b4fc48a0".to_string(),
        );
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_hex_or_dec_param(&req, "token_id").unwrap();
        assert_eq!(
            s,
            "0x06f5b84a20c71f393a8b5b4d74f62df2017b80bb5dcef199e2fa6952b4fc48a0"
        );
    }

    #[test]
    fn test_get_query_string_param_hex_or_dec_invalid() {
        let mut params = HashMap::new();
        params.insert("token_id".to_string(), "255".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_hex_or_dec_param(&req, "other_param");
        assert!(s.is_err());

        let mut params = HashMap::new();
        params.insert("token_id".to_string(), "0xajfojigehih".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = get_query_string_hex_or_dec_param(&req, "token_id");
        assert!(s.is_err());
    }
}

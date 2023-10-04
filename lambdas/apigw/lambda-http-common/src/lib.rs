use lambda_http::{Body, Error, Request, RequestExt, Response};
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
            Ok(v)
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

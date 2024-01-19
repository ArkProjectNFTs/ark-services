pub mod params;
pub use params::*;

pub mod format;
pub mod lambda_context;
pub use lambda_context::LambdaCtx;

use ark_dynamodb::ProviderError;
use ark_sqlx::providers::ProviderError as SqlxProviderError;
use lambda_http::{http::header, Body, Error, Response};
use serde::Serialize;

/// Generic response returned from any http lambda.
#[derive(Debug, Serialize)]
pub struct ArkApiResponse<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    pub result: T,
    // To be extended as needed.
}

/// Generic response from Lambdas.
#[derive(Debug)]
pub struct LambdaHttpResponse {
    pub capacity: f64,
    pub inner: Response<Body>,
}

/// Generic errors for http parsing.
#[derive(Debug, thiserror::Error)]
pub enum LambdaHttpError {
    #[error("Bad param")]
    ParamParsing(String),
    #[error("Missing param")]
    ParamMissing(String),
    // TODO: to be merged in a ark-common crate.
    #[error(transparent)]
    Provider(ProviderError),
    #[error(transparent)]
    SqlxProvider(SqlxProviderError),
}

impl From<ProviderError> for LambdaHttpError {
    fn from(e: ProviderError) -> Self {
        LambdaHttpError::Provider(e)
    }
}

impl From<SqlxProviderError> for LambdaHttpError {
    fn from(e: SqlxProviderError) -> Self {
        LambdaHttpError::SqlxProvider(e)
    }
}

impl TryFrom<LambdaHttpError> for Response<Body> {
    type Error = Error;

    fn try_from(e: LambdaHttpError) -> Result<Self, Self::Error> {
        Ok(Response::builder()
            .status(400)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(match e {
                LambdaHttpError::ParamParsing(s) => s.into(),
                LambdaHttpError::ParamMissing(s) => s.into(),
                LambdaHttpError::Provider(s) => s.to_string().into(),
                LambdaHttpError::SqlxProvider(s) => s.to_string().into(),
            })
            .map_err(Box::new)?)
    }
}

/// Returns a `Response` with OK status and the given body serialized as a JSON.
pub fn ok_body_rsp<T: Serialize>(body: &T) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS")
        .header(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Content-Type, Authorization",
        )
        .body(serde_json::to_string(&body)?.into())
        .map_err(Box::new)?)
}

/// Returns a `Response` with NOT_FOUND status.
pub fn not_found_rsp() -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(404)
        .header(header::CONTENT_TYPE, "text/plain")
        .body("".into())
        .map_err(Box::new)?)
}

/// Returns a `Reponse` with BAD_REQUEST status and the given message as body.
pub fn bad_request_rsp(message: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(400)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(message.into())
        .map_err(Box::new)?)
}

/// Returns a `Reponse` with INTERNAL_SERVER_ERROR status and the given message as body.
pub fn internal_server_error_rsp(message: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(500)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(message.into())
        .map_err(Box::new)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::*;
    use lambda_http::{Request, RequestExt};
    use std::collections::HashMap;

    #[test]
    fn test_is_hexadecimal_with_prefix() {
        let s = "0x123";
        assert!(is_hexadecimal_with_prefix(s));
    }

    #[test]
    fn test_is_hexadecimal_with_prefix_invalid() {
        let s = "0x";
        assert!(!is_hexadecimal_with_prefix(s));

        let s = "1234";
        assert!(!is_hexadecimal_with_prefix(s));
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

        let s = require_string_param(&req, "address", HttpParamSource::QueryString).unwrap();
        assert_eq!(s, "0x1234");

        let s = require_string_param(&req, "other_param", HttpParamSource::QueryString);
        assert!(s.is_err());
    }

    #[test]
    fn test_get_path_param() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "0x1234".to_string());
        let req = Request::default().with_path_parameters(params.clone());

        let s = require_string_param(&req, "address", HttpParamSource::Path).unwrap();
        assert_eq!(s, "0x1234");

        let s = require_string_param(&req, "other_param", HttpParamSource::Path);
        assert!(s.is_err());
    }

    #[test]
    fn test_get_query_string_param_hex() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "0x1234".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = require_hex_param(&req, "address", HttpParamSource::QueryString).unwrap();
        assert_eq!(s, format!("0x{:064x}", 0x1234));
    }

    #[test]
    fn test_get_path_param_hex() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "0x1234".to_string());
        let req = Request::default().with_path_parameters(params.clone());

        let s = require_hex_param(&req, "address", HttpParamSource::Path).unwrap();
        assert_eq!(s, format!("0x{:064x}", 0x1234));
    }

    #[test]
    fn test_get_query_string_param_hex_invalid() {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "1234".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = require_hex_param(&req, "address", HttpParamSource::QueryString);
        assert!(s.is_err());

        let mut params = HashMap::new();
        params.insert("address".to_string(), "jfeoehguoirehgo".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = require_hex_param(&req, "address", HttpParamSource::QueryString);
        assert!(s.is_err());
    }

    #[test]
    fn test_get_query_string_param_hex_or_dec() {
        let mut params = HashMap::new();
        params.insert("token_id".to_string(), "255".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = require_hex_or_dec_param(&req, "token_id", HttpParamSource::QueryString).unwrap();
        assert_eq!(s, format!("0x{:064x}", 0xff));

        let mut params = HashMap::new();
        params.insert(
            "token_id".to_string(),
            "0x6f5b84a20c71f393a8b5b4d74f62df2017b80bb5dcef199e2fa6952b4fc48a0".to_string(),
        );
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = require_hex_or_dec_param(&req, "token_id", HttpParamSource::QueryString).unwrap();
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

        let s = require_hex_or_dec_param(&req, "other_param", HttpParamSource::QueryString);
        assert!(s.is_err());

        let mut params = HashMap::new();
        params.insert("token_id".to_string(), "0xajfojigehih".to_string());
        let req = Request::default().with_query_string_parameters(params.clone());

        let s = require_hex_or_dec_param(&req, "token_id", HttpParamSource::QueryString);
        assert!(s.is_err());
    }
}

use actix_web::dev::ServiceRequest;
use actix_web::{error, Error};
use actix_web::{error::ResponseError, HttpResponse};
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct UnauthorizedError;

impl fmt::Display for UnauthorizedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unauthorized")
    }
}

impl ResponseError for UnauthorizedError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized().json("Unauthorized")
    }
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    println!("Validating credentials");

    let user = std::env::var("API_USER").unwrap_or_default();
    let password = std::env::var("API_PASSWORD").unwrap_or_default();

    if credentials.user_id() == user && credentials.password().unwrap_or_default() == password {
        Ok(req)
    } else {
        Err((error::ErrorUnauthorized("Unauthorized"), req))
    }
}

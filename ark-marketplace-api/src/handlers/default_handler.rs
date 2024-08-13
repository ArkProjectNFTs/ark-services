use crate::types::default::{HealthCheckResponse, HealthCheckResponseV1};
use actix_web::get;
use actix_web::{HttpResponse, Responder};

#[utoipa::path(
    responses(
        (status = 200, description = "Health Check", body = HealthCheckResponse)
    )
)]
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    responses(
        (status = 200, description = "Health Check", body = HealthCheckResponse)
    )
)]
#[get("/")]
pub async fn root() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    context_path = "/v1",
    responses(
        (status = 200, description = "Health Check", body = HealthCheckResponseV1)
    )
)]
#[get("/health")]
pub async fn health_check_v1() -> impl Responder {
    HttpResponse::Ok().json(HealthCheckResponseV1 {
        status_v1: "okV1".to_string(),
    })
}

use actix_web::{HttpResponse, Responder};
use utoipa::ToSchema;
use serde::Serialize;
use actix_web::get;
use utoipa::OpenApi;


#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        root
    ),
    components(schemas(HealthCheckResponse))
)]
pub(super) struct HealthApi;

#[derive(ToSchema, Serialize)]
struct HealthCheckResponse {
    #[schema()]
    status: String,
}

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

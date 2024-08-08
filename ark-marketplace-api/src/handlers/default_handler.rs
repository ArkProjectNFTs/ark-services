use actix_web::{HttpResponse, Responder};
use actix_web::web::{Json, Path};
use actix_web::Error;
use apistos::actix::CreatedJson;
use apistos::{api_operation, ApiComponent};
use schemars::JsonSchema;


#[derive(serde::Serialize, JsonSchema)]
struct HealthCheckResponse {
    status: String,
}

#[api_operation(summary = "Health Check")]
async fn health_check() -> Result<CreatedJson<HealthCheckResponse>, Error> {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "ok".to_string(),
    })
}

use crate::db::default_query::get_last_sales;
use crate::types::default::{HealthCheckResponse, HealthCheckResponseV1};
use actix_web::{get, web};
use actix_web::{HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

#[utoipa::path(
    tag = "HealthCheck",
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

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get the 12 last sales", body = LastSalesResponse),
        (status = 400, description = "Data not found", body = String),
    )
)]
#[get("/last-sales")]
pub async fn last_sales(db_pools: web::Data<Arc<[PgPool; 2]>>) -> impl Responder {
    let db_access = &db_pools[0];
    match get_last_sales(db_access).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(data) => HttpResponse::Ok().json(json!({
            "data": data,
        })),
        Err(err) => {
            tracing::error!("error query last_sales: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check).service(root).service(last_sales);
}

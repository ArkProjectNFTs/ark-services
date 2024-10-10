use crate::db::default_query::{get_last_sales, get_live_auctions, get_trending};
use crate::models::token::get_event_types;
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

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get event filters", body = LastSalesResponse),
        (status = 400, description = "Data not found", body = String),
    )
)]
#[get("/event-filters")]
pub async fn event_filters() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "data": get_event_types(),
    }))
}

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get the 6 last live auctions", body = LiveAuctionsResponse),
        (status = 400, description = "Data not found", body = String),
    )
)]
#[get("/live-auctions")]
pub async fn live_auctions(db_pools: web::Data<Arc<[PgPool; 2]>>) -> impl Responder {
    let db_access = &db_pools[0];
    match get_live_auctions(db_access).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(data) => HttpResponse::Ok().json(json!({
            "data": data,
        })),
        Err(err) => {
            tracing::error!("error query live_auctions: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get the 6 last live auctions", body = TrendingResponse),
        (status = 400, description = "Data not found", body = String),
    )
)]
#[get("/trending")]
pub async fn trending(db_pools: web::Data<Arc<[PgPool; 2]>>) -> impl Responder {
    let db_access = &db_pools[0];
    // if we need later we can pass the timerange parameter to the url.
    const TIME_RANGE: &str = "7d";
    match get_trending(db_access, TIME_RANGE).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(data) => HttpResponse::Ok().json(json!({
            "data": data,
        })),
        Err(err) => {
            tracing::error!("error query trending: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        .service(root)
        .service(last_sales)
        .service(live_auctions)
        .service(trending)
        .service(event_filters);
}

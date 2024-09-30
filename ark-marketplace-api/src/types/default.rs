use crate::models::default::{LastSale, LiveAuction, Trending};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize)]
pub struct HealthCheckResponseV1 {
    #[schema()]
    pub status_v1: String,
}

#[derive(ToSchema, Serialize)]
pub struct HealthCheckResponse {
    #[schema()]
    pub status: String,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct LastSalesResponse {
    data: Vec<LastSale>,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct LiveAuctionsResponse {
    data: Vec<LiveAuction>,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct TrendingResponse {
    data: Vec<Trending>,
}

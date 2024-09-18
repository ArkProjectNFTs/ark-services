use crate::models::portfolio::{OfferApiData, StatsData};
use crate::models::token::{TokenPortfolioActivityData, TokenPortfolioData};
use serde::Serialize;

#[derive(utoipa::ToSchema, Serialize)]
pub struct TokensPortfolioResponse {
    data: Vec<TokenPortfolioData>,
    token_count: i64,
    next_page: i64,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct PortfolioOffersResponse {
    data: Vec<OfferApiData>,
    token_count: i64,
    next_page: i64,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct PortfolioActivityResponse {
    data: Vec<TokenPortfolioActivityData>,
    token_count: i64,
    next_page: i64,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct PortfolioStatsResponse {
    data: StatsData,
}

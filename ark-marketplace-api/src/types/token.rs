use crate::models::token::{
    TokenActivityData, TokenData, TokenInformationData, TokenMarketData, TokenOfferOneData,
};
use serde::Serialize;

#[derive(utoipa::ToSchema, Serialize)]
pub struct TokensResponse {
    data: Vec<TokenData>,
    token_count: i64,
    next_page: i64,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct TokenResponse {
    data: Vec<TokenInformationData>,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct TokenMarketDataResponse {
    data: Vec<TokenMarketData>,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct TokenOffersResponse {
    data: Vec<TokenOfferOneData>,
    count: i64,
    next_page: i64,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct TokenActivitiesResponse {
    data: Vec<TokenActivityData>,
    count: i64,
    next_page: i64,
}

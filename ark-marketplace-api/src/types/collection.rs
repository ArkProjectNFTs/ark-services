use crate::models::collection::{
    CollectionActivityData, CollectionData, CollectionPortfolioData, CollectionSearchData,
    OwnerData,
};
use serde::Serialize;

#[derive(utoipa::ToSchema, Serialize)]
pub struct CollectionResponse {
    data: Vec<CollectionData>,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct CollectionActivityResponse {
    data: Vec<CollectionActivityData>,
    #[schema(value_type = String, example = "777")]
    collection_count: i64,
    #[schema(value_type = String, example = "3")]
    next_page: i64,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct CollectionPortfolioResponse {
    data: Vec<CollectionPortfolioData>,
    #[schema(value_type = String, example = "777")]
    collection_count: i64,
    #[schema(value_type = String, example = "3")]
    next_page: i64,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct CollectionSearchResponse {
    collections: Vec<CollectionSearchData>,
    accounts: Vec<OwnerData>,
}

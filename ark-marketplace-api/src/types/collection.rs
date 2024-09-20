use crate::models::collection::{
    CollectionActivityData, CollectionData, CollectionFullData, CollectionPortfolioData,
    CollectionSearchData, OwnerData,
};
use serde::Serialize;
use std::collections::HashMap;

#[derive(utoipa::ToSchema, Serialize)]
pub struct CollectionResponse {
    data: CollectionData,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct CollectionsResponse {
    data: Vec<CollectionFullData>,
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

#[derive(Serialize, utoipa::ToSchema)]
pub struct AttributesResponse {
    pub data: HashMap<String, AttributeValues>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AttributeValues {
    pub values: HashMap<String, usize>,
}

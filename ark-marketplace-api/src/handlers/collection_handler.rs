use super::utils::extract_page_params;
use super::utils::CHAIN_ID;
use crate::db::query::{
    get_collection_activity_data, get_collection_data, get_collections_data,
    get_portfolio_collections_data, search_collections_data,
};
use crate::managers::elasticsearch_manager::ElasticsearchManager;
use crate::models::token::TokenEventType;
use crate::utils::http_utils::normalize_address;
use actix_web::get;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct CollectionQueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    time_range: Option<String>,
    sort: Option<String>,
    direction: Option<String>,
}

#[derive(Deserialize)]
pub struct PortfolioCollectionQueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
}

#[derive(Deserialize, Debug)]
struct ActivityQueryParameters {
    direction: Option<String>,
    types: Option<Vec<TokenEventType>>,
}

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get collections", body = CollectionsResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
        ("sort" = Option<String>, Query, description = "Field to sort by, e.g., 'floor_price', 'floor_percentage', 'volume', 'top_bid', 'number_of_sales', 'marketcap', 'listed'"),
        ("direction" = Option<String>, Query, description = "Direction to sort by, 'asc' or 'desc'")
    )
)]
#[get("/collections")]
pub async fn get_collections(
    query_params: web::Query<CollectionQueryParameters>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let page = query_params.page.unwrap_or(1);
    let items_per_page = query_params.items_per_page.unwrap_or(100);
    let time_range = query_params.time_range.as_deref().unwrap_or("1D");
    let sort = query_params.sort.as_deref().unwrap_or("volume");
    let direction = query_params.direction.as_deref().unwrap_or("desc");

    let db_access = &db_pools[0];
    match get_collections_data(db_access, page, items_per_page, time_range, sort, direction).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok((collections_data, has_next_page, count)) => HttpResponse::Ok().json(json!({
            "data": collections_data,
            "count": count,
            "next_page": if has_next_page { Some(page + 1) } else { None }
        })),
        Err(err) => {
            tracing::error!("error query get_collections: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get collection data", body = CollectionResponse),
        (status = 400, description = "Data not found", body = String),
    )
)]
#[get("/collections/{contract_address}/{chain_id}")]
pub async fn get_collection(
    path: web::Path<(String, String)>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
    redis_con: web::Data<Arc<Mutex<MultiplexedConnection>>>,
) -> impl Responder {
    let (contract_address, chain_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let db_access = &db_pools[0];
    let mut redis_con_ref = redis_con.get_ref().lock().await;
    match get_collection_data(
        db_access,
        &mut redis_con_ref,
        &normalized_address,
        &chain_id,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(collection_data) => HttpResponse::Ok().json(json!({
            "data": collection_data,
        })),
        Err(err) => {
            tracing::error!("error query get_collection: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get collection activity", body = CollectionActivityResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
    )
)]
#[get("/collections/{contract_address}/activity")]
pub async fn get_collection_activity(
    req: HttpRequest,
    path: web::Path<String>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let contract_address = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let (page, items_per_page) = match extract_page_params(req.query_string(), 1, 100) {
        Err(msg) => return HttpResponse::BadRequest().json(msg),
        Ok((page, items_per_page)) => (page, items_per_page),
    };

    let params = serde_qs::from_str::<ActivityQueryParameters>(req.query_string());
    if let Err(e) = params {
        let msg = format!("Error when parsing query parameters: {}", e);
        tracing::error!(msg);
        return HttpResponse::BadRequest().json(msg);
    }
    let params = params.unwrap();
    let direction = params.direction.as_deref().unwrap_or("desc");

    let db_access = &db_pools[0];

    match get_collection_activity_data(
        db_access,
        &normalized_address,
        CHAIN_ID,
        page,
        items_per_page,
        direction,
        &params.types,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok((collection_data, has_next_page, collection_count)) => HttpResponse::Ok().json(json!({
            "data": collection_data,
            "collection_count": collection_count,
            "next_page": if has_next_page { Some(page + 1) } else { None }
        })),
        Err(err) => {
            tracing::error!("error query get_collection_activity: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[utoipa::path(
    tag = "Portfolio",
    responses(
        (status = 200, description = "Get portfolio collections", body = CollectionPortfolioResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
    )
)]
#[get("/portfolio/{user_address}/collections")]
pub async fn get_portfolio_collections(
    query_parameters: web::Query<PortfolioCollectionQueryParameters>,
    path: web::Path<String>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let page = query_parameters.page.unwrap_or(1);
    let items_per_page = query_parameters.items_per_page.unwrap_or(100);
    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);

    let db_access = &db_pools[0];
    match get_portfolio_collections_data(db_access, &normalized_address, page, items_per_page).await
    {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok((collection_data, has_next_page, collection_count)) => HttpResponse::Ok().json(json!({
            "data": collection_data,
            "collection_count": collection_count,
            "next_page": if has_next_page { Some(page + 1) } else { None }
        })),
        Err(err) => {
            tracing::error!("error query get_portfolio_collections_data: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    limit: Option<i64>,
}

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Search in a collection", body = CollectionSearchResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("q" = String, Query, description = "Can be a starknetId or a starknet user address"),
    )
)]
#[get("/collections/search")]
pub async fn search_collections(
    query_parameters: web::Query<SearchQuery>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let query_search = query_parameters.q.as_deref();
    let db_access = &db_pools[0];
    let items = query_parameters.limit.unwrap_or(8);

    match search_collections_data(
        db_access,
        query_search.unwrap_or("").to_lowercase().as_str(),
        items,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok((collection_data, owner_data)) => HttpResponse::Ok().json(json!({
        "data": {
            "collections": collection_data,
            "accounts": owner_data
        }
        })),
        Err(err) => {
            tracing::error!("error query search_collections_data: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[utoipa::path(
    tag = "Collections",
    responses(
        (status = 200, description = "Get traits in a collection", body = AttributesResponse),
        (status = 400, description = "Data not found", body = String),
    )
)]
#[get("/collections/{address}/traits")]
pub async fn get_traits(
    path: web::Path<String>,
    es_data: web::Data<HashMap<String, String>>,
) -> impl Responder {
    let contract_address = path.into_inner();
    let elasticsearch_manager = ElasticsearchManager::new(es_data.get_ref().clone());

    let normalized_address = normalize_address(&contract_address);
    let result = elasticsearch_manager
        .get_attributes_for_collection(&normalized_address, CHAIN_ID)
        .await;

    match result {
        Ok(json_response) => HttpResponse::Ok().json(json!({
            "data": json_response
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to retrieve data: {}", e)
        })),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_collections)
        .service(get_traits)
        .service(get_collection_activity)
        .service(get_collection)
        .service(get_portfolio_collections)
        .service(search_collections);
}

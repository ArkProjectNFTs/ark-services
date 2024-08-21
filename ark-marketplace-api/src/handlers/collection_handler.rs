use super::utils::extract_page_params;
use super::utils::CHAIN_ID;
use crate::db::db_access::DatabaseAccess;
use crate::db::query::{
    get_collection_activity_data, get_collection_data, get_collections_data,
    get_portfolio_collections_data, search_collections_data,
};
use crate::models::token::TokenEventType;
use crate::utils::http_utils::normalize_address;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct CollectionQueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    time_range: Option<String>,
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

pub async fn get_collections<D: DatabaseAccess + Sync>(
    query_parameters: web::Query<CollectionQueryParameters>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let page = query_parameters.page.unwrap_or(1);
    let items_per_page = query_parameters.items_per_page.unwrap_or(100);
    let time_range = query_parameters.time_range.as_deref().unwrap_or("");

    let db_access = db_pool.get_ref();
    match get_collections_data(db_access, page, items_per_page, time_range).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(collection_data) => HttpResponse::Ok().json(collection_data),
        Err(err) => {
            tracing::error!("error query get_collections: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_collection<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
    redis_con: web::Data<Arc<Mutex<MultiplexedConnection>>>,
) -> impl Responder {
    let (contract_address, chain_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let db_access = db_pool.get_ref();
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

pub async fn get_collection_activity<D: DatabaseAccess + Sync>(
    req: HttpRequest,
    path: web::Path<String>,
    db_pool: web::Data<D>,
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

    let db_access = db_pool.get_ref();

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

pub async fn get_portfolio_collections<D: DatabaseAccess + Sync>(
    query_parameters: web::Query<PortfolioCollectionQueryParameters>,
    path: web::Path<String>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let page = query_parameters.page.unwrap_or(1);
    let items_per_page = query_parameters.items_per_page.unwrap_or(100);
    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);

    let db_access = db_pool.get_ref();
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

pub async fn search_collections<D: DatabaseAccess + Sync>(
    query_parameters: web::Query<SearchQuery>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let query_search = query_parameters.q.as_deref();
    let db_access = db_pool.get_ref();
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

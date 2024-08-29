use super::utils::CHAIN_ID;
use crate::db::db_access::DatabaseAccess;
use crate::db::query::{
    flush_all_data_query, get_collection_floor_price, get_token_activity_data, get_token_data,
    get_token_marketdata, get_token_offers_data, get_tokens_data, get_tokens_portfolio_data,
    get_tokens_data_by_id,
};
use crate::managers::elasticsearch_manager::ElasticsearchManager;
use crate::models::token::TokenEventType;
use crate::models::token::TokenOfferOneData;
use crate::utils::currency_utils::compute_floor_difference;
use crate::utils::http_utils::normalize_address;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use serde_json::json;
use serde_qs;
use std::sync::Arc;
use tokio::sync::Mutex;
use urlencoding::decode;
use serde_urlencoded;
use super::utils::extract_page_params;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct QueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    buy_now: Option<String>,
    sort: Option<String>,
    direction: Option<String>,
    collection: Option<String>,
    disable_cache: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ActivityQueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    direction: Option<String>,
    types: Option<Vec<TokenEventType>>,
    // range ?
}

fn extract_query_params(
    query_parameters: &web::Query<QueryParameters>,
) -> (i64, i64, bool, &str, &str) {
    let page = query_parameters.page.unwrap_or(1);
    let items_per_page = query_parameters.items_per_page.unwrap_or(100);
    let buy_now = query_parameters.buy_now.as_deref() == Some("true");
    let sort = query_parameters.sort.as_deref().unwrap_or("price");
    let direction = query_parameters.direction.as_deref().unwrap_or("asc");
    (page, items_per_page, buy_now, sort, direction)
}

pub async fn get_tokens<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    query_parameters: web::Query<QueryParameters>,
    db_pool: web::Data<D>,
    redis_con: web::Data<Arc<Mutex<MultiplexedConnection>>>,
) -> impl Responder {
    let page = query_parameters.page.unwrap_or(1);
    let items_per_page = query_parameters.items_per_page.unwrap_or(100);
    let (contract_address, chain_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);
    let buy_now = query_parameters.buy_now.as_deref() == Some("true");
    let sort = query_parameters.sort.as_deref().unwrap_or("price");
    let direction = query_parameters.direction.as_deref().unwrap_or("asc");
    let disable_cache = query_parameters.disable_cache.as_deref() == Some("true");

    let db_access = db_pool.get_ref();
    let mut redis_con_ref = redis_con.get_ref().lock().await;
    match get_tokens_data(
        db_access,
        &mut redis_con_ref,
        &normalized_address,
        &chain_id,
        page,
        items_per_page,
        buy_now,
        sort,
        direction,
        disable_cache,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok((ref collection_data, _, _)) if collection_data.is_empty() => {
            HttpResponse::NotFound().body("data not found")
        }
        Ok((collection_data, _has_next_page, token_count)) => HttpResponse::Ok().json(json!({
            "data": collection_data,
            "token_count": token_count,
            "next_page": /*if has_next_page { Some(page + 1) } else { None }*/ page + 1
        })),
        Err(err) => {
            tracing::error!("error query get_tokens_data: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_token<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let db_access = db_pool.get_ref();
    match get_token_data(db_access, &normalized_address, &chain_id, &token_id).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(token_data) => HttpResponse::Ok().json(json!({
            "data": token_data,
        })),
        Err(err) => {
            tracing::error!("error query get_tokens_data: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_token_market<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let db_access = db_pool.get_ref();
    match get_token_marketdata(db_access, &normalized_address, &chain_id, &token_id).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(token_data) => HttpResponse::Ok().json(json!({
            "data": token_data,
        })),
        Err(err) => {
            tracing::error!("error query get_tokens_data: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_tokens_portfolio<D: DatabaseAccess + Sync>(
    path: web::Path<String>,
    query_parameters: web::Query<QueryParameters>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (page, items_per_page, buy_now, sort, direction) = extract_query_params(&query_parameters);
    let collection = query_parameters.collection.as_deref().unwrap_or("");

    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);

    let db_access = db_pool.get_ref();

    match get_tokens_portfolio_data(
        db_access,
        &normalized_address,
        page,
        items_per_page,
        buy_now,
        sort,
        direction,
        collection,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok((collection_data, has_next_page, token_count)) => HttpResponse::Ok().json(json!({
            "data": collection_data,
            "token_count": token_count,
            "next_page": if has_next_page { Some(page + 1) } else { None }
        })),
        Err(err) => {
            tracing::error!("error query portfolio token: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_token_offers<D: DatabaseAccess + Sync>(
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let (page, items_per_page) = match extract_page_params(req.query_string(), 1, 100) {
        Err(msg) => return HttpResponse::BadRequest().json(msg),
        Ok((page, items_per_page)) => (page, items_per_page),
    };

    let db_access = db_pool.get_ref();
    let floor_price = match get_collection_floor_price(db_access, &normalized_address, &chain_id)
        .await
    {
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("data not found"),
        Ok(floor_price) => floor_price,
        Err(err) => {
            tracing::error!("error query get_collection_floor_price: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let (token_offers_data, has_next_page, count) = match get_token_offers_data(
        db_access,
        &normalized_address,
        &chain_id,
        &token_id,
        page,
        items_per_page,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("data not found"),
        Ok((token_offers_data, has_next_page, count)) => (token_offers_data, has_next_page, count),
        Err(err) => {
            tracing::error!("error query get_token_offers_data: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let token_offers_data: Vec<TokenOfferOneData> = token_offers_data
        .iter()
        .map(|data| TokenOfferOneData {
            offer_id: data.offer_id,
            price: data.amount.clone(), // TODO: handle currency conversion
            source: data.source.clone(),
            expire_at: data.expire_at,
            hash: data.hash.clone(),
            floor_difference: compute_floor_difference(
                data.amount.clone(),
                data.currency_address.clone(),
                floor_price.value.clone(),
            ),
        })
        .collect();
    HttpResponse::Ok().json(json!({
        "data": token_offers_data,
        "count": count,
        "next_page": if has_next_page { Some(page + 1)} else { None}
    }))
}

pub async fn get_token_activity<D: DatabaseAccess + Sync>(
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);
    let db_access = db_pool.get_ref();

    let params = serde_qs::from_str::<ActivityQueryParameters>(req.query_string());
    if let Err(e) = params {
        let msg = format!("Error when parsing query parameters: {}", e);
        tracing::error!(msg);
        return HttpResponse::BadRequest().json(msg);
    }
    let params = params.unwrap();

    let page = params.page.unwrap_or(1);
    let items_per_page = params.items_per_page.unwrap_or(100);
    let direction = params.direction.as_deref().unwrap_or("desc");
    let (token_activity_data, has_next_page, count) = match get_token_activity_data(
        db_access,
        &normalized_address,
        &chain_id,
        &token_id,
        page,
        items_per_page,
        direction,
        &params.types,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("data not found"),
        Ok((token_activity_data, has_next_page, count)) => {
            (token_activity_data, has_next_page, count)
        }
        Err(err) => {
            tracing::error!("error query get_token_activity_data: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(json!({
        "data": token_activity_data,
        "next_page": if has_next_page { Some(page + 1)} else { None },
        "count": count,
    }))
}

pub async fn get_token_trait_filters<D: DatabaseAccess + Sync>(
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

pub async fn get_filtered_tokens<D: DatabaseAccess + Sync>(
    req: HttpRequest,
    path: web::Path<String>,
    db_pool: web::Data<D>,
    es_data: web::Data<HashMap<String, String>>,
) -> impl Responder {
    let query_string = req.query_string();
    let query_params: HashMap<String, String> = serde_urlencoded::from_str(query_string).unwrap_or_default();
    let contract_address = path.into_inner();
    let normalized_address = normalize_address(&contract_address);
    let buy_now = query_params.get("buy_now").map(String::as_str) == Some("true");
    let sort = query_params.get("sort").map(String::as_str).unwrap_or("price");
    let direction = query_params.get("direction").map(String::as_str).unwrap_or("asc");


    let (page, items_per_page) = match extract_page_params(query_string, 1, 100) {
        Err(msg) => return HttpResponse::BadRequest().json(msg),
        Ok((page, items_per_page)) => (page, items_per_page),
    };


    let mut token_ids = None;
    if let Some(traits_param) = query_params.get("traits") {
        let decoded_traits = decode(traits_param).expect("Failed to decode traits");
        let traits_map: HashMap<String, Vec<String>> = serde_json::from_str(&decoded_traits).expect("Failed to parse JSON");

        let elasticsearch_manager = ElasticsearchManager::new(es_data.get_ref().clone());

        let result = elasticsearch_manager
            .search_tokens_by_traits(&normalized_address, CHAIN_ID, traits_map)
            .await;

        token_ids = match result {
            Ok(token_ids) => Some(token_ids),
            Err(e) => return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to retrieve data: {}", e)
            })),
        };
    }

    let db_access = db_pool.get_ref();

    match get_tokens_data_by_id(
           db_access,
           &normalized_address,
           CHAIN_ID,
           page,
           items_per_page,
           buy_now,
           sort,
           direction,
           token_ids,
       )
       .await
       {
           Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
           Ok((ref collection_data, _, _)) if collection_data.is_empty() => {
               HttpResponse::NotFound().body("data not found")
           }
           Ok((collection_data, _has_next_page, token_count)) => HttpResponse::Ok().json(json!({
               "data": collection_data,
               "token_count": token_count,
               "next_page": page + 1
           })),
           Err(err) => {
               tracing::error!("error query get_tokens_data: {}", err);
               HttpResponse::InternalServerError().finish()
           }
       }
}

pub async fn flush_all_data<D: DatabaseAccess + Sync>(db_pool: web::Data<D>) -> impl Responder {
    let db_access = db_pool.get_ref();
    match flush_all_data_query(db_access).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

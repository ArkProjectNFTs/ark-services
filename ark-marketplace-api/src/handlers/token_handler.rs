use super::utils::extract_page_params;
use super::utils::CHAIN_ID;
use crate::db::db_access::DatabaseAccess;
use crate::db::query::{
    flush_all_data_query, get_collection_floor_price, get_token_activity_data, get_token_data,
    get_token_marketdata, get_token_offers_data, get_tokens_data, get_tokens_portfolio_data,
    refresh_token_metadata,
};
use crate::managers::elasticsearch_manager::ElasticsearchManager;
use crate::models::token::TokenOfferOneData;
use crate::models::token::{TokenEventType, TokenInformationData};
use crate::utils::currency_utils::compute_floor_difference;
use crate::utils::http_utils::normalize_address;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use serde_json::json;
use serde_qs;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;
use urlencoding::decode;

#[derive(Deserialize)]
pub struct QueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    buy_now: Option<String>,
    sort: Option<String>,
    direction: Option<String>,
    collection: Option<String>,
    disable_cache: Option<String>,
    filters: Option<String>,
    sort_value: Option<String>,
    search: Option<String>,
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

#[utoipa::path(
    tag = "Tokens",
    responses(
        (status = 200, description = "Get tokens", body = TokensResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("address" = String, Path, description = "The contract address of the collection"),
        ("chain_id" = String, Path, description = "The blockchain chain ID"),

        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
        ("search" = Option<String>, Query, description = "Filter by token id"),
        ("buy_now" = Option<String>, Query, description = "Filter tokens by 'buy now' status"),
        ("sort" = Option<String>, Query, description = "Sort field, defaults to 'price'"),
        ("direction" = Option<String>, Query, description = "Sort direction, 'asc' or 'desc', defaults to 'asc'"),
        ("sort_value" = Option<String>, Query, description = "Specific value for sorting, used to refine results")
    )
)]
#[get("/collections/{address}/{chain_id}/tokens")]
pub async fn get_tokens(
    path: web::Path<(String, String)>,
    query_parameters: web::Query<QueryParameters>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
    redis_con: web::Data<Arc<Mutex<MultiplexedConnection>>>,
    es_data: web::Data<HashMap<String, String>>,
) -> impl Responder {
    let page = query_parameters.page.unwrap_or(1);
    let items_per_page = query_parameters.items_per_page.unwrap_or(100);
    let (contract_address, chain_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);
    let buy_now = query_parameters.buy_now.as_deref() == Some("true");
    let sort = query_parameters.sort.as_deref().unwrap_or("price");
    let direction = query_parameters.direction.as_deref().unwrap_or("asc");
    let search = query_parameters.search.as_deref().unwrap_or("");
    let mut disable_cache = query_parameters.disable_cache.as_deref() == Some("true");
    let sort_value = query_parameters
        .sort_value
        .as_deref()
        .map(|s| s.to_string());
    // disable cache
    if sort_value.is_some() {
        disable_cache = true;
    }
    let db_access = &db_pools[0];
    let mut redis_con_ref = redis_con.get_ref().lock().await;
    let mut token_ids = None;
    let mut token_id = None;

    match search.parse::<String>() {
        Ok(parsed_token_id) => {
            disable_cache = true;
            token_id = Some(parsed_token_id)
        }
        Err(_) => {
            error!("get_tokens: error parsing search field");
        }
    }

    if let Some(filters_param) = &query_parameters.filters {
        if !filters_param.is_empty() {
            let decoded_filters = decode(filters_param).expect("Failed to decode filters");
            let filters_map: HashMap<String, serde_json::Value> =
                serde_json::from_str(&decoded_filters).expect("Failed to parse JSON");

            if let Some(traits) = filters_map.get("traits") {
                // for now we dont want to cache results with traits
                disable_cache = true;
                let traits_map: HashMap<String, Vec<String>> =
                    serde_json::from_value(traits.clone()).expect("Failed to parse traits JSON");

                let elasticsearch_manager = ElasticsearchManager::new(es_data.get_ref().clone());

                let result = elasticsearch_manager
                    .search_tokens_by_traits(&normalized_address, CHAIN_ID, traits_map)
                    .await;

                token_ids = match result {
                    Ok(token_ids) => Some(token_ids),
                    Err(e) => {
                        return HttpResponse::InternalServerError().json(json!({
                            "error": format!("Failed to retrieve data: {}", e)
                        }))
                    }
                };
            }
        }
    }

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
        sort_value,
        disable_cache,
        token_ids,
        token_id,
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

#[utoipa::path(
    tag = "Tokens",
    responses(
        (status = 200, description = "Get a token information", body = TokensResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("address" = String, Path, description = "The contract address of the collection"),
        ("chain_id" = String, Path, description = "The blockchain chain ID"),
        ("token_id" = String, Path, description = "The token ID"),
    )
)]
#[get("/tokens/{address}/{chain_id}/{token_id}")]
pub async fn get_token(
    path: web::Path<(String, String, String)>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let db_access = &db_pools[0];
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

#[utoipa::path(
    tag = "Tokens",
    responses(
        (status = 200, description = "Get a token marketdata", body = TokenMarketDataResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("address" = String, Path, description = "The contract address of the collection"),
        ("chain_id" = String, Path, description = "The blockchain chain ID"),
        ("token_id" = String, Path, description = "The token ID"),
    )
)]
#[get("/tokens/{address}/{chain_id}/{token_id}/marketdata")]
pub async fn get_token_market(
    path: web::Path<(String, String, String)>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let db_access = &db_pools[0];
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

#[utoipa::path(
    tag = "Portfolio",
    responses(
        (status = 200, description = "Get tokens in a portfolio", body = TokenOffersResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("user_address" = String, Path, description = "The user address"),

        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
    )
)]
#[get("/portfolio/{user_address}")]
pub async fn get_tokens_portfolio(
    path: web::Path<String>,
    query_parameters: web::Query<QueryParameters>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let (page, items_per_page, buy_now, sort, direction) = extract_query_params(&query_parameters);
    let collection = query_parameters.collection.as_deref().unwrap_or("");

    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);

    let db_access = &db_pools[0];

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

#[utoipa::path(
    tag = "Tokens",
    responses(
        (status = 200, description = "Get token offers", body = TokenOffersResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("address" = String, Path, description = "The contract address of the collection"),
        ("chain_id" = String, Path, description = "The blockchain chain ID"),
        ("token_id" = String, Path, description = "The token ID"),

        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
    )
)]
#[get("/tokens/{address}/{chain_id}/{token_id}/offers")]
pub async fn get_token_offers(
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let (page, items_per_page) = match extract_page_params(req.query_string(), 1, 100) {
        Err(msg) => return HttpResponse::BadRequest().json(msg),
        Ok((page, items_per_page)) => (page, items_per_page),
    };

    let db_access = &db_pools[0];
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

#[utoipa::path(
    tag = "Tokens",
    responses(
        (status = 200, description = "Get token activities", body = TokenActivitiesResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("address" = String, Path, description = "The contract address of the collection"),
        ("chain_id" = String, Path, description = "The blockchain chain ID"),
        ("token_id" = String, Path, description = "The token ID"),

        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
    )
)]
#[get("/tokens/{address}/{chain_id}/{token_id}/activity")]
pub async fn get_token_activity(
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);
    let db_access = &db_pools[0];

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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct RefreshMetadataRequest {
    pub contract_address: String,
    pub token_id: String,
}

fn is_metadata_refreshing(token_data: &TokenInformationData) -> bool {
    if let Some(metadata_status) = &token_data.metadata_status {
        if metadata_status == "TO_REFRESH" {
            return true;
        }
    }

    if let Some(metadata_updated_at) = token_data.metadata_updated_at {
        let current_time = Utc::now().timestamp();
        let time_diff = current_time - metadata_updated_at;
        if time_diff <= 60 {
            return true;
        }
    }

    false
}

#[utoipa::path(
    tag = "Tokens",
    responses(
        (status = 200, description = "Metadata refresh has been requested", body = String),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("contract_address" = String, Path, description = "The contract address of the collection"),
        ("token_id" = String, Path, description = "The token ID"),
    )
)]
#[post("/metadata/refresh")]
pub async fn post_refresh_token_metadata(
    body: web::Json<RefreshMetadataRequest>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let db_access = &db_pools[1];
    let normalized_address = normalize_address(&body.contract_address);

    match get_token_data(db_access, &normalized_address, CHAIN_ID, &body.token_id).await {
        Err(e) => {
            error!("error: {:?}", e);
            HttpResponse::NotFound().json(json!({
                "message": "Token does not exist"
            }))
        }
        Ok(token_data) => {
            if is_metadata_refreshing(&token_data) {
                return HttpResponse::Ok().json(json!({
                    "message": "Metadata refresh has already been requested"
                }));
            }

            match refresh_token_metadata(db_access, &normalized_address, CHAIN_ID, &body.token_id)
                .await
            {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "message": "Metadata refresh has been requested"
                })),
                Err(err) => {
                    tracing::error!("Failed to refresh metadata: {}", err);
                    HttpResponse::InternalServerError().json(json!({
                        "message": "Failed to refresh metadata"
                    }))
                }
            }
        }
    }
}

pub async fn flush_all_data<D: DatabaseAccess + Sync>(
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let db_access = &db_pools[0];
    match flush_all_data_query(db_access).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tokens)
        .service(get_token)
        .service(get_token_market)
        .service(get_tokens_portfolio)
        .service(get_token_offers)
        .service(get_token_activity)
        .service(post_refresh_token_metadata);
}

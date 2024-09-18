use super::utils::extract_page_params;
use super::utils::CHAIN_ID;
use crate::db::portfolio_query::{get_activity_data, get_offers_data, get_stats_data};
use crate::models::portfolio::OfferApiData;
use crate::models::token::TokenEventType;
use crate::types::offer_type::OfferType;
use crate::utils::currency_utils::compute_floor_difference;
use crate::utils::http_utils::normalize_address;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use serde_qs;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
struct ActivityQueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    direction: Option<String>,
    types: Option<Vec<TokenEventType>>,
}

#[utoipa::path(
    tag = "Portfolio",
    responses(
        (status = 200, description = "Get activity for a portfolio", body = PortfolioActivityResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("user_address" = String, Path, description = "Address of the user"),

        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
    )
)]
#[get("/portfolio/{user_address}/activity")]
pub async fn get_activity(
    req: HttpRequest,
    path: web::Path<String>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);
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
    let (token_activity_data, has_next_page, count) = match get_activity_data(
        db_access,
        CHAIN_ID,
        &normalized_address,
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

#[utoipa::path(
    tag = "Portfolio",
    responses(
        (status = 200, description = "Get offers for a portfolio", body = PortfolioOffersResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("user_address" = String, Path, description = "Address of the user"),

        ("type" = Option<String>, Query, description = "'made' or 'received' to filter either made or received offers"),
        ("page" = Option<i32>, Query, description = "Page number for pagination, defaults to 1"),
        ("items_per_page" = Option<i32>, Query, description = "Number of items per page, defaults to 100"),
    )
)]
#[get("/portfolio/{user_address}/offers")]
pub async fn get_offers(
    req: HttpRequest,
    path: web::Path<String>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);
    let db_access = &db_pools[0];

    let (page, items_per_page) = match extract_page_params(req.query_string(), 1, 100) {
        Err(msg) => return HttpResponse::BadRequest().json(msg),
        Ok((page, items_per_page)) => (page, items_per_page),
    };

    let query_string = req.query_string();
    let query_params: HashMap<String, String> = serde_urlencoded::from_str(query_string).unwrap();

    let type_offer_str = query_params.get("type").map(|s| s.as_str()).unwrap_or("");
    let type_offer = match OfferType::from_str(type_offer_str) {
        Ok(t) => t,
        Err(_) => return HttpResponse::BadRequest().json("Invalid type"),
    };

    let (token_offers_data, has_next_page, count) = match get_offers_data(
        db_access,
        CHAIN_ID,
        &normalized_address,
        page,
        items_per_page,
        type_offer,
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

    let token_offers_data: Vec<OfferApiData> = token_offers_data
        .iter()
        .map(|data| OfferApiData {
            offer_id: data.offer_id,
            price: data.amount.clone(), // TODO: handle currency conversion
            from_address: data.source.clone(),
            currency_address: data.currency_address.clone(),
            to_address: data.to_address.clone(),
            expire_at: data.expire_at,
            hash: data.hash.clone(),
            token_id: data.token_id.clone(),
            floor_difference: compute_floor_difference(
                data.amount.clone(),
                data.currency_address.clone(),
                data.collection_floor_price.clone(),
            ),
            collection_address: data.collection_address.clone(),
            collection_name: data.collection_name.clone(),
            is_verified: data.is_verified,
            metadata: data.metadata.clone(),
        })
        .collect();

    HttpResponse::Ok().json(json!({
        "data": token_offers_data,
        "next_page": if has_next_page { Some(page + 1)} else { None },
        "count": count,
    }))
}

#[utoipa::path(
    tag = "Portfolio",
    responses(
        (status = 200, description = "Get stats for a portfolio", body = PortfolioStatsResponse),
        (status = 400, description = "Data not found", body = String),
    ),
    params(
        ("user_address" = String, Path, description = "Address of the user"),
    )
)]
#[get("/portfolio/{user_address}/stats")]
pub async fn get_stats(
    path: web::Path<String>,
    db_pools: web::Data<Arc<[PgPool; 2]>>,
) -> impl Responder {
    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);
    let db_access = &db_pools[0];

    let stats_data = match get_stats_data(db_access, CHAIN_ID, &normalized_address).await {
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("data not found"),
        Ok(stats_data) => stats_data,
        Err(err) => {
            tracing::error!("error query get_stats: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(json!({
        "data": stats_data,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_activity)
        .service(get_offers)
        .service(get_stats);
}

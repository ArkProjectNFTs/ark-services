use crate::db::db_access::DatabaseAccess;
use crate::db::query::{
    get_collection_floor_price, get_token_data, get_token_offers_data, get_tokens_data,
    get_tokens_portfolio_data,
};
use crate::models::token::TokenOfferOneData;
use crate::utils::currency_utils::compute_floor_difference;
use crate::utils::http_utils::normalize_address;
use actix_web::{web, HttpResponse, Responder};
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    path: web::Path<(String, String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (contract_address, chain_id, token_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

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

    let token_offers_data =
        match get_token_offers_data(db_access, &normalized_address, &chain_id, &token_id).await {
            Err(sqlx::Error::RowNotFound) => {
                return HttpResponse::NotFound().body("data not found")
            }
            Ok(token_offers_data) => token_offers_data,
            Err(err) => {
                tracing::error!("error query get_token_offers_data: {}", err);
                return HttpResponse::InternalServerError().finish();
            }
        };
    let token_offers_data: Vec<TokenOfferOneData> = token_offers_data
        .iter()
        .map(|data| TokenOfferOneData {
            offer_id: data.offer_id,
            price: data.amount.clone(),
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
    }))
}

#[cfg(test)]
mod tests {
    use crate::db::db_access::MockDb;
    use crate::handlers::collection_handler::get_collections;
    use crate::models::collection::CollectionData;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_get_tokens_handler() {
        let app = test::init_service(App::new().app_data(web::Data::new(MockDb)).route(
            "/collection/0x/tokens",
            web::get().to(get_collections::<MockDb>),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/collection/0x/tokens")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let tokens_data: Vec<CollectionData> = serde_json::from_slice(&response_body).unwrap();

        assert_eq!(tokens_data[0].id, Some("1".to_string()));
        assert_eq!(tokens_data[0].address, Some(4));
    }
}

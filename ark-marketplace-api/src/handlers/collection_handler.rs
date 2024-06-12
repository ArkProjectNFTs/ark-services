use crate::db::db_access::DatabaseAccess;
use crate::db::query::{get_collection_data, get_collections_data, get_portfolio_collections_data};
use crate::utils::http_utils::normalize_address;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

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
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_collection<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (contract_address, chain_id) = path.into_inner();
    let normalized_address = normalize_address(&contract_address);

    let db_access = db_pool.get_ref();
    match get_collection_data(db_access, &normalized_address, &chain_id).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(collection_data) => HttpResponse::Ok().json(collection_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
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
        Ok(collection_data) => HttpResponse::Ok().json(collection_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use crate::db::db_access::MockDb;
    use crate::handlers::collection_handler::get_collections;
    use crate::models::collection::CollectionData;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_get_collections_handler() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(MockDb))
                .route("/collections", web::get().to(get_collections::<MockDb>)),
        )
        .await;

        let req = test::TestRequest::get().uri("/collections").to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let collection_data: Vec<CollectionData> = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(
            collection_data[0].image,
            Some("https://example.com/image.png".to_string())
        );
        assert_eq!(
            collection_data[0].collection_name,
            Some("Example Collection".to_string())
        );
        assert_eq!(collection_data[0].floor, Some("1".to_string()));
        assert_eq!(collection_data[0].floor_7d_percentage, Some(4));
        assert_eq!(collection_data[0].volume_7d_eth, Some(789));
        assert_eq!(collection_data[0].top_offer, Some("Top Offer".to_string()));
        assert_eq!(collection_data[0].sales_7d, Some(10));
        assert_eq!(collection_data[0].marketcap, Some(1112));
        assert_eq!(collection_data[0].listed_items, Some(13));
        assert_eq!(collection_data[0].listed_percentage, Some(14));
    }
}

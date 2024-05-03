use crate::db::db_access::DatabaseAccess;
use crate::db::query::get_collection_data;
use actix_web::{web, HttpResponse, Responder};

pub async fn get_collection<D: DatabaseAccess + Sync>(
    path: web::Path<String>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let contract_address = path.into_inner();

    let db_access = db_pool.get_ref();
    match get_collection_data(db_access, &contract_address).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(collection_data) => HttpResponse::Ok().json(collection_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use crate::db::db_access::MockDb;
    use crate::handlers::token_handler::get_collection_data;
    use crate::models::token::{TokenData, TokenWithHistory, TokenWithOffers};
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_get_collection_handler() {
        let app = test::init_service(App::new().app_data(web::Data::new(MockDb)).route(
            "/collection/{contract_address}/",
            web::get().to(get_collection::<MockDb>),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/collection/0xABCDEF123456")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let collection_data: CollectionData = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(collection_data.image, "https://example.com/image.png");
        assert_eq!(collection_data.collection_name, "Example Collection");
        assert_eq!(collection_data.floor, 1.23);
        assert_eq!(collection_data.floor_7d_percentage, 4.56);
        assert_eq!(collection_data.volume_7d_eth, 789.0);
        assert_eq!(collection_data.top_offer, Some("Top Offer".to_string()));
        assert_eq!(collection_data.sales_7d, 10);
        assert_eq!(collection_data.marketcap, 1112.0);
        assert_eq!(collection_data.listed_items, 13);
        assert_eq!(collection_data.listed_percentage, 14.0);
    }
}

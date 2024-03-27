use crate::db::db_access::DatabaseAccess;
use crate::db::query::{
    delete_token_data, flush_all_data_query, get_token_by_collection_data, get_token_data,
    get_token_history_data, get_token_offers_data, get_tokens_by_account_data,
};
use crate::utils::http_utils::convert_param_to_hex;
use actix_web::{web, HttpResponse, Responder};

pub async fn get_token<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();

    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match get_token_data(db_access, &token_address, &token_id_hex).await {
                Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
                Ok(token_data) => HttpResponse::Ok().json(token_data),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_tokens_by_collection<D: DatabaseAccess + Sync>(
    path: web::Path<String>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let token_address = path.into_inner();
    let db_access = db_pool.get_ref();
    match get_token_by_collection_data(db_access, &token_address).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(token_data) => HttpResponse::Ok().json(token_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_token_history<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();
    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match get_token_history_data(db_access, &token_address, &token_id_hex).await {
                Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
                Ok(token_data) => HttpResponse::Ok().json(token_data),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_token_offers<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();
    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match get_token_offers_data(db_access, &token_address, &token_id_hex).await {
                Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
                Ok(token_data) => HttpResponse::Ok().json(token_data),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_tokens_by_account<D: DatabaseAccess + Sync>(
    path: web::Path<String>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let owner = path.into_inner();
    let db_access = db_pool.get_ref();
    match get_tokens_by_account_data(db_access, owner.as_str()).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(token_data) => HttpResponse::Ok().json(token_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_token_context<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();
    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match delete_token_data(db_access, &token_address, &token_id_hex).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn flush_all_data<D: DatabaseAccess + Sync>(db_pool: web::Data<D>) -> impl Responder {
    let db_access = db_pool.get_ref();
    match flush_all_data_query(db_access).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use crate::db::db_access::MockDb;
    use crate::handlers::token_handler::{
        get_token, get_token_history, get_token_offers, get_tokens_by_account,
        get_tokens_by_collection,
    };
    use crate::models::token::{TokenData, TokenWithHistory, TokenWithOffers};
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_get_token_handler() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(MockDb))
                .route("/token/{address}/{id}", web::get().to(get_token::<MockDb>)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/token/0xABCDEF123456/0xABCDEF123456")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let token_data: TokenData = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(token_data.token_chain_id, "chainXYZ");
        assert_eq!(token_data.token_address, "0xABCDEF123456");
        assert_eq!(token_data.token_id, "789");
        assert_eq!(token_data.listed_timestamp, 1234567890);
        assert_eq!(token_data.updated_timestamp, 1234567891);
        assert_eq!(token_data.current_owner, Some("owner123".to_string()));
        assert_eq!(token_data.last_price, Some("100".to_string()));
        assert_eq!(token_data.quantity, Some("10".to_string()));
        assert_eq!(token_data.start_amount, Some("50".to_string()));
        assert_eq!(token_data.end_amount, Some("150".to_string()));
        assert_eq!(token_data.start_date, Some(1234567890));
        assert_eq!(token_data.end_date, Some(1234567891));
        assert_eq!(token_data.is_listed, Some(true));
        assert_eq!(token_data.has_offer, Some(false));
        assert_eq!(token_data.broker_id, Some("brokerXYZ".to_string()));
    }

    #[actix_rt::test]
    async fn test_get_tokens_by_collection_handler() {
        let app = test::init_service(App::new().app_data(web::Data::new(MockDb)).route(
            "/tokens/collection/{address}",
            web::get().to(get_tokens_by_collection::<MockDb>),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/tokens/collection/0xABCDEF123456")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let tokens: Vec<TokenData> = serde_json::from_slice(&response_body).unwrap();

        assert_eq!(tokens.len(), 2);

        let token1 = &tokens[0];
        assert_eq!(token1.token_chain_id, "chainXYZ");
        assert_eq!(token1.token_address, "0xABCDEF123456");
        assert_eq!(token1.token_id, "789");
        assert_eq!(token1.listed_timestamp, 1234567890);
        assert_eq!(token1.updated_timestamp, 1234567891);
        assert_eq!(token1.current_owner, Some("owner123".to_string()));
        assert_eq!(token1.last_price, Some("100".to_string()));
        assert_eq!(token1.quantity, Some("10".to_string()));
        assert_eq!(token1.start_amount, Some("50".to_string()));
        assert_eq!(token1.end_amount, Some("150".to_string()));
        assert_eq!(token1.start_date, Some(1234567890));
        assert_eq!(token1.end_date, Some(1234567891));
        assert_eq!(token1.is_listed, Some(true));
        assert_eq!(token1.has_offer, Some(false));
        assert_eq!(token1.broker_id, Some("brokerXYZ".to_string()));

        let token2 = &tokens[1];
        assert_eq!(token2.token_chain_id, "chainWXYZ");
        assert_eq!(token2.token_address, "0xABCDEF1234567");
        assert_eq!(token2.token_id, "7890");
        assert_eq!(token2.listed_timestamp, 1234567890);
        assert_eq!(token2.updated_timestamp, 1234567891);
        assert_eq!(token2.current_owner, Some("owner1234".to_string()));
        assert_eq!(token2.last_price, Some("100".to_string()));
        assert_eq!(token2.quantity, Some("10".to_string()));
        assert_eq!(token2.start_amount, Some("50".to_string()));
        assert_eq!(token2.end_amount, Some("150".to_string()));
        assert_eq!(token2.start_date, Some(1234567890));
        assert_eq!(token2.end_date, Some(1234567891));
        assert_eq!(token2.is_listed, Some(true));
        assert_eq!(token2.has_offer, Some(false));
        assert_eq!(token2.broker_id, Some("brokerXYZ".to_string()));
    }

    #[actix_rt::test]
    async fn test_get_token_history_handler() {
        let app = test::init_service(App::new().app_data(web::Data::new(MockDb)).route(
            "/token/{address}/{id}/history",
            web::get().to(get_token_history::<MockDb>),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/token/0xABCDEF123456/1234/history")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = test::read_body(resp).await;
        let token_history: TokenWithHistory = serde_json::from_slice(&response_body).unwrap();

        assert_eq!(token_history.token_address, "0xABCDEF123456");
        assert_eq!(token_history.token_id, "789");
        assert_eq!(token_history.current_owner, Some("owner123".to_string()));
        assert_eq!(token_history.last_price, Some("100".to_string()));
        assert_eq!(token_history.history.len(), 1);
        assert_eq!(token_history.history[0].event_type, "Listing");
        assert_eq!(token_history.history[0].event_timestamp, 1234567890);
        assert_eq!(token_history.history[0].order_status, "Active");
        assert_eq!(
            token_history.history[0].new_owner,
            Some("owner123".to_string())
        );
        assert_eq!(token_history.history[0].amount, Some("100".to_string()));
    }

    #[actix_rt::test]
    async fn test_get_token_offers_handler() {
        let app = test::init_service(App::new().app_data(web::Data::new(MockDb)).route(
            "/token/{address}/{id}/offers",
            web::get().to(get_token_offers::<MockDb>),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/token/0xABCDEF123456/789/offers")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = test::read_body(resp).await;
        let token_offers: TokenWithOffers = serde_json::from_slice(&response_body).unwrap();

        assert_eq!(token_offers.token_address, "0xABCDEF123456");
        assert_eq!(token_offers.token_id, "789");
        assert_eq!(token_offers.current_owner, Some("owner123".to_string()));
        assert_eq!(token_offers.last_price, Some("100".to_string()));
        assert_eq!(token_offers.offers.len(), 1);
        assert_eq!(token_offers.offers[0].offer_maker, "maker123");
        assert_eq!(token_offers.offers[0].offer_amount, "100");
        assert_eq!(token_offers.offers[0].offer_quantity, "10");
        assert_eq!(token_offers.offers[0].offer_timestamp, 1234567890);
    }

    #[actix_rt::test]
    async fn test_get_tokens_data() {
        let app = test::init_service(App::new().app_data(web::Data::new(MockDb)).route(
            "/tokens/{owner}",
            web::get().to(get_tokens_by_account::<MockDb>),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/tokens/owner123")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let tokens: Vec<TokenData> = serde_json::from_slice(&response_body).unwrap();

        assert_eq!(tokens.len(), 2);

        assert_eq!(tokens[0].token_chain_id, "chainXYZ");
        assert_eq!(tokens[0].token_address, "0xABCDEF123456");
        assert_eq!(tokens[0].token_id, "789");
        assert_eq!(tokens[0].listed_timestamp, 1234567890);
        assert_eq!(tokens[0].updated_timestamp, 1234567891);
        assert_eq!(tokens[0].current_owner, Some("owner123".to_string()));
        assert_eq!(tokens[0].last_price, Some("100".to_string()));
        assert_eq!(tokens[0].quantity, Some("10".to_string()));
        assert_eq!(tokens[0].start_amount, Some("50".to_string()));
        assert_eq!(tokens[0].end_amount, Some("150".to_string()));
        assert_eq!(tokens[0].start_date, Some(1234567890));
        assert_eq!(tokens[0].end_date, Some(1234567891));
        assert_eq!(tokens[0].broker_id, Some("brokerXYZ".to_string()));

        assert_eq!(tokens[1].token_chain_id, "chainWXYZ");
        assert_eq!(tokens[1].token_address, "0xABCDEF1234567");
        assert_eq!(tokens[1].token_id, "7890");
        assert_eq!(tokens[1].listed_timestamp, 2234567890);
        assert_eq!(tokens[1].updated_timestamp, 2234567891);
        assert_eq!(tokens[1].current_owner, Some("owner1234".to_string()));
        assert_eq!(tokens[1].last_price, Some("200".to_string()));
        assert_eq!(tokens[1].quantity, Some("20".to_string()));
        assert_eq!(tokens[1].start_amount, Some("100".to_string()));
        assert_eq!(tokens[1].end_amount, Some("300".to_string()));
        assert_eq!(tokens[1].start_date, Some(2234567890));
        assert_eq!(tokens[1].end_date, Some(2234567891));
        assert_eq!(tokens[1].broker_id, Some("brokerWXYZ".to_string()));
    }
}

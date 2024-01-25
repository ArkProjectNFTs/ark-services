use actix_web::{web, HttpResponse, Responder};
use crate::db::db_access::DatabaseAccess;
use crate::db::query::{get_token_data, get_token_by_collection_data};

pub async fn get_token<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {

    let (token_address, token_id) = path.into_inner();
    let db_access = db_pool.get_ref();

    match get_token_data(db_access, &token_address, &token_id).await {
        Ok(token_data) => HttpResponse::Ok().json(token_data),
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
        Ok(token_data) => HttpResponse::Ok().json(token_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, http};
    use crate::db::db_access::MockDb;
    use crate::handlers::token_handler::{get_token, get_tokens_by_collection};
    use crate::models::token::TokenData;

    #[actix_rt::test]
    async fn test_get_token_handler() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(MockDb))
                .route("/token/{address}/{id}", web::get().to(get_token::<MockDb>)),
        ).await;

        let req = test::TestRequest::get().uri("/token/0xABCDEF123456/token789").to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let token_data: TokenData = serde_json::from_slice(&response_body).unwrap();
        assert_eq!(token_data.token_chain_id, "chainXYZ");
        assert_eq!(token_data.token_address, "0xABCDEF123456");
        assert_eq!(token_data.token_id, "token789");
        assert_eq!(token_data.listed_timestamp, 1234567890);
        assert_eq!(token_data.updated_timestamp, 1234567891);
        assert_eq!(token_data.current_owner, "owner123");
        assert_eq!(token_data.current_price, Some("100".to_string()));
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
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(MockDb))
                .route("/tokens/collection/{address}", web::get().to(get_tokens_by_collection::<MockDb>)),
        ).await;

        let req = test::TestRequest::get().uri("/tokens/collection/0xABCDEF123456").to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let tokens: Vec<TokenData> = serde_json::from_slice(&response_body).unwrap();

        assert_eq!(tokens.len(), 2);

        let token1 = &tokens[0];
        assert_eq!(token1.token_chain_id, "chainXYZ");
        assert_eq!(token1.token_address, "0xABCDEF123456");
        assert_eq!(token1.token_id, "token789");
        assert_eq!(token1.listed_timestamp, 1234567890);
        assert_eq!(token1.updated_timestamp, 1234567891);
        assert_eq!(token1.current_owner, "owner123");
        assert_eq!(token1.current_price, Some("100".to_string()));
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
        assert_eq!(token2.token_id, "token7890");
        assert_eq!(token2.listed_timestamp, 1234567890);
        assert_eq!(token2.updated_timestamp, 1234567891);
        assert_eq!(token2.current_owner, "owner1234");
        assert_eq!(token2.current_price, Some("100".to_string()));
        assert_eq!(token2.quantity, Some("10".to_string()));
        assert_eq!(token2.start_amount, Some("50".to_string()));
        assert_eq!(token2.end_amount, Some("150".to_string()));
        assert_eq!(token2.start_date, Some(1234567890));
        assert_eq!(token2.end_date, Some(1234567891));
        assert_eq!(token2.is_listed, Some(true));
        assert_eq!(token2.has_offer, Some(false));
        assert_eq!(token2.broker_id, Some("brokerXYZ".to_string()));
    }
}


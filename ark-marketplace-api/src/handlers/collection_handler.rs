use crate::db::db_access::DatabaseAccess;
use crate::db::query::{
    delete_migrations_query, delete_token_data, flush_all_data_query, get_token_by_collection_data,
    get_token_data, get_token_history_data, get_token_offers_data, get_tokens_by_account_data,
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
}

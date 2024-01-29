use actix_web::{web, HttpResponse, Responder};
use crate::db::db_access::DatabaseAccess;
use crate::db::query::get_token_data;

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

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, http};
    use crate::db::db_access::MockDb;
    use crate::handlers::token_handler::get_token;
    use crate::models::token::TokenData;

    #[actix_rt::test]
    async fn test_get_token_handler() {
        let _ = env_logger::builder().is_test(true).try_init();

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
        assert_eq!(token_data.token_id, "token789");
    }
}

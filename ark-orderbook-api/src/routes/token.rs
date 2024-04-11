use crate::routes::auth::validator;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;

use crate::handlers::token_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::basic(validator);

    cfg.service(
        web::scope("/token")
            .wrap(auth)
            .route(
                "/{address}/{id}",
                web::get().to(token_handler::get_token::<PgPool>),
            )
            .route(
                "/{address}/{id}/history",
                web::get().to(token_handler::get_token_history::<PgPool>),
            )
            .route(
                "/{address}/{id}/offers",
                web::get().to(token_handler::get_token_offers::<PgPool>),
            )
            .route(
                "/{token_address}/{token_id}",
                web::delete().to(token_handler::delete_token_context::<PgPool>),
            ),
    )
    .service(
        web::scope("/tokens")
            .route(
                "/collection/{collection_id}",
                web::get().to(token_handler::get_tokens_by_collection::<PgPool>),
            )
            .route(
                "/{owner}",
                web::get().to(token_handler::get_tokens_by_account::<PgPool>),
            ),
    )
    .route(
        "/flush-all-data",
        web::delete().to(token_handler::flush_all_data::<PgPool>),
    );
}

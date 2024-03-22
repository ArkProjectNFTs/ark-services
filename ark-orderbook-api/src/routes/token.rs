use actix_web::web;
use sqlx::PgPool;

use crate::handlers::token_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/token/{address}/{id}",
        web::get().to(token_handler::get_token::<PgPool>),
    );

    cfg.route(
        "/tokens/collection/{collection_id}",
        web::get().to(token_handler::get_tokens_by_collection::<PgPool>),
    );

    cfg.route(
        "/token/{address}/{id}/history",
        web::get().to(token_handler::get_token_history::<PgPool>),
    );

    cfg.route(
        "/token/{address}/{id}/offers",
        web::get().to(token_handler::get_token_offers::<PgPool>),
    );

    cfg.route(
        "/tokens/{owner}",
        web::get().to(token_handler::get_tokens_by_account::<PgPool>),
    );

    cfg.route(
        "/token/{token_address}/{token_id}",
        web::delete().to(token_handler::delete_token_context::<PgPool>),
    );
}

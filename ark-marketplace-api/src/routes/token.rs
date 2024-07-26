use crate::handlers::token_handler::{get_token, get_token_market, get_token_offers, get_tokens};
use crate::routes::auth::validator;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;

use crate::handlers::token_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::basic(validator);

    cfg.route(
        "/collections/{address}/{chain_id}/tokens",
        web::get().to(get_tokens::<PgPool>),
    );

    cfg.route(
        "/tokens/{address}/{chain_id}/{token_id}",
        web::get().to(get_token::<PgPool>),
    );

    cfg.route(
        "/tokens/{address}/{chain_id}/{token_id}/marketdata",
        web::get().to(get_token_market::<PgPool>),
    );

    cfg.route(
        "/tokens/{address}/{chain_id}/{token_id}/offers",
        web::get().to(get_token_offers::<PgPool>),
    );

    cfg.route(
        "/tokens/{address}/{chain_id}/{token_id}/activity",
        web::get().to(token_handler::get_token_activity::<PgPool>),
    );

    cfg.route(
        "/flush-all-data",
        web::delete()
            .to(token_handler::flush_all_data::<PgPool>)
            .wrap(auth.clone()),
    );
}

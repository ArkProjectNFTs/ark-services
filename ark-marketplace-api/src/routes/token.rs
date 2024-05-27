use actix_web::web;
use sqlx::PgPool;

use crate::handlers::token_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/collection/{address}/tokens",
        web::get().to(token_handler::get_tokens::<PgPool>),
    );
}

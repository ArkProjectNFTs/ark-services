use actix_web::web;
use sqlx::PgPool;

use crate::handlers::token_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/collections/{address}/{chain_id}/tokens",
        web::get().to(token_handler::get_tokens::<PgPool>),
    );

    cfg.route(
        "/portfolio/{user_address}",
        web::get().to(token_handler::get_tokens_portfolio::<PgPool>),
    );
}

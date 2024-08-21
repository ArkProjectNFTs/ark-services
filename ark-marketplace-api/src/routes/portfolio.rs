
use actix_web::web;
use sqlx::PgPool;

use crate::handlers::{portfolio_handler, token_handler};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/portfolio/{user_address}/activity",
        web::get().to(portfolio_handler::get_activity::<PgPool>),
    );

    cfg.route(
        "/portfolio/{user_address}",
        web::get().to(token_handler::get_tokens_portfolio::<PgPool>),
    );
}

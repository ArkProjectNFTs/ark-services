use actix_web::web;
use sqlx::PgPool;

use crate::handlers::token_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/token/{address}/{id}",
        web::get().to(token_handler::get_token::<PgPool>),
    );
}

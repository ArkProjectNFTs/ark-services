use actix_web::web;
use sqlx::PgPool;

use crate::handlers::collection_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/collections",
        web::get().to(collection_handler::get_collections::<PgPool>),
    );

    cfg.route(
        "/collections/{address}",
        web::get().to(collection_handler::get_collection::<PgPool>),
    );

    cfg.route(
        "/portfolio/{user_address}/collections",
        web::get().to(collection_handler::get_portfolio_collections::<PgPool>),
    );
}

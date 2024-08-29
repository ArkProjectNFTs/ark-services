use actix_web::web;
use sqlx::PgPool;

use crate::handlers::collection_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/collections",
        web::get().to(collection_handler::get_collections::<PgPool>),
    );

    cfg.route(
        "/collections/{address}/activity",
        web::get().to(collection_handler::get_collection_activity::<PgPool>),
    );

    cfg.route(
        "/collections/{address}/{chain_id}",
        web::get().to(collection_handler::get_collection::<PgPool>),
    );

    cfg.route(
        "/collections/{address}/traits",
        web::get().to(collection_handler::get_traits::<PgPool>),
    );

    cfg.route(
        "/collections/{address}/filters",
        web::get().to(collection_handler::get_filtered_tokens::<PgPool>),
    );

    cfg.route(
        "/portfolio/{user_address}/collections",
        web::get().to(collection_handler::get_portfolio_collections::<PgPool>),
    );

    cfg.route(
        "/collections/search",
        web::get().to(collection_handler::search_collections::<PgPool>),
    );
}

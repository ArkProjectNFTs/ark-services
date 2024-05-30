use actix_web::web;
use sqlx::PgPool;

use crate::handlers::collection_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/collections").route(
        "",
        web::get().to(collection_handler::get_collections::<PgPool>),
    ));

    cfg.route(
        "/collection/{address}/{chain_id}",
        web::get().to(collection_handler::get_collection::<PgPool>),
    );
}

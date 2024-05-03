use crate::routes::auth::validator;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;

use crate::handlers::collection_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::basic(validator);
    cfg.service(web::scope("/collections").route(
        "",
        web::get().to(collection_handler::get_collection::<PgPool>),
    ));
}

use crate::routes::auth::validator;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;

use crate::handlers::collection_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::basic(validator);
    cfg.service(web::scope("/collection/{address}/tokens").route(
        "",
        web::get().to(token_handler::get_tokens::<PgPool>),
    ));
}

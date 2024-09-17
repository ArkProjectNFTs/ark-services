use crate::handlers::token_handler;
use crate::routes::auth::validator;
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::basic(validator);

    cfg.route(
        "/flush-all-data",
        web::delete()
            .to(token_handler::flush_all_data::<PgPool>)
            .wrap(auth.clone()),
    );
}

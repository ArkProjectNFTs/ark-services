use actix_web::web;

use crate::handlers::default_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(default_handler::health_check));

    cfg.route("/", web::get().to(default_handler::health_check));
}

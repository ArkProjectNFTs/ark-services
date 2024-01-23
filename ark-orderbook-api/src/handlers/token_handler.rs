use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::db::query::get_token_data;

pub async fn get_token(
    path: web::Path<(String, String)>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();

    match get_token_data(&db_pool, &token_address, &token_id).await {
        Ok(token_data) => HttpResponse::Ok().json(token_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

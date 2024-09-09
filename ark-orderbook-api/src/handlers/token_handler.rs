use crate::db::db_access::DatabaseAccess;
use crate::db::query::{
    delete_migrations_query, delete_token_data, flush_all_data_query, get_token_by_collection_data,
    get_token_data, get_token_history_data, get_token_offers_data, get_tokens_by_account_data,
};
use crate::utils::http_utils::convert_param_to_hex;
use actix_web::{web, HttpResponse, Responder};
use tracing::error;

pub async fn get_token<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();

    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match get_token_data(db_access, &token_address, &token_id_hex).await {
                Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
                Ok(token_data) => HttpResponse::Ok().json(token_data),
                Err(err) => {
                    eprintln!("error get_token_data: {}", err);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(err) => {
            error!("error convert_param_to_hex: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_tokens_by_collection<D: DatabaseAccess + Sync>(
    path: web::Path<String>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let token_address = path.into_inner();
    let db_access = db_pool.get_ref();
    match get_token_by_collection_data(db_access, &token_address).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(token_data) => HttpResponse::Ok().json(token_data),
        Err(err) => {
            eprintln!("error get_tokens_by_collection: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_token_history<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();
    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match get_token_history_data(db_access, &token_address, &token_id_hex).await {
                Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
                Ok(token_data) => HttpResponse::Ok().json(token_data),
                Err(err) => {
                    eprintln!("error get_token_history_data: {}", err);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(err) => {
            eprintln!("error convert_param_to_hex: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_token_offers<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();
    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match get_token_offers_data(db_access, &token_address, &token_id_hex).await {
                Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
                Ok(token_data) => HttpResponse::Ok().json(token_data),
                Err(err) => {
                    eprintln!("error get_token_offers_data: {}", err);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(err) => {
            eprintln!("error convert_param_to_hex: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_tokens_by_account<D: DatabaseAccess + Sync>(
    path: web::Path<String>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let owner = path.into_inner();
    let db_access = db_pool.get_ref();
    match get_tokens_by_account_data(db_access, owner.as_str()).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok(token_data) => HttpResponse::Ok().json(token_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_token_context<D: DatabaseAccess + Sync>(
    path: web::Path<(String, String)>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let (token_address, token_id) = path.into_inner();
    match convert_param_to_hex(&token_id) {
        Ok(token_id_hex) => {
            let db_access = db_pool.get_ref();
            match delete_token_data(db_access, &token_address, &token_id_hex).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn flush_all_data<D: DatabaseAccess + Sync>(db_pool: web::Data<D>) -> impl Responder {
    let db_access = db_pool.get_ref();
    match flush_all_data_query(db_access).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_migrations<D: DatabaseAccess + Sync>(db_pool: web::Data<D>) -> impl Responder {
    let db_access = db_pool.get_ref();
    match delete_migrations_query(db_access).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

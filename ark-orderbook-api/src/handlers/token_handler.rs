use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    status: String,
    message: String,
}

pub async fn get_token(path: web::Path<(String, String)>) -> impl Responder {
    let (address, id) = path.into_inner();
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Operation completed successfully".to_string(),
    };
    HttpResponse::Ok().json(response)
}

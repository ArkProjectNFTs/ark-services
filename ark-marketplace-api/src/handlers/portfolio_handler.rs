use crate::db::portfolio_db_access::DatabaseAccess;
use crate::db::portfolio_query::get_activity_data;
use crate::models::token::TokenEventType;
use crate::utils::http_utils::normalize_address;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use serde_qs;

const CHAIN_ID: &str = "0x534e5f4d41494e";

#[derive(Deserialize, Debug)]
struct ActivityQueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    direction: Option<String>,
    types: Option<Vec<TokenEventType>>,
}

pub async fn get_activity<D: DatabaseAccess + Sync>(
    req: HttpRequest,
    path: web::Path<String>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let user_address = path.into_inner();
    let normalized_address = normalize_address(&user_address);
    let db_access = db_pool.get_ref();

    let params = serde_qs::from_str::<ActivityQueryParameters>(req.query_string());
    if let Err(e) = params {
        let msg = format!("Error when parsing query parameters: {}", e);
        tracing::error!(msg);
        return HttpResponse::BadRequest().json(msg);
    }
    let params = params.unwrap();

    let page = params.page.unwrap_or(1);
    let items_per_page = params.items_per_page.unwrap_or(100);
    let direction = params.direction.as_deref().unwrap_or("desc");
    let (token_activity_data, has_next_page, count) = match get_activity_data(
        db_access,
        CHAIN_ID,
        &normalized_address,
        page,
        items_per_page,
        direction,
        &params.types,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("data not found"),
        Ok((token_activity_data, has_next_page, count)) => {
            (token_activity_data, has_next_page, count)
        }
        Err(err) => {
            tracing::error!("error query get_token_activity_data: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(json!({
        "data": token_activity_data,
        "next_page": if has_next_page { Some(page + 1)} else { None },
        "count": count,
    }))
}

use crate::db::db_access::DatabaseAccess;
use crate::db::query::get_tokens_data;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct QueryParameters {
    page: Option<i64>,
    items_per_page: Option<i64>,
    buy_now: Option<String>,
    sort: Option<String>,
    direction: Option<String>,
}

pub async fn get_tokens<D: DatabaseAccess + Sync>(
    path: web::Path<String>,
    query_parameters: web::Query<QueryParameters>,
    db_pool: web::Data<D>,
) -> impl Responder {
    let page = query_parameters.page.unwrap_or(1);
    let items_per_page = query_parameters.items_per_page.unwrap_or(100);
    let contract_address = path.into_inner();
    let buy_now = query_parameters.buy_now.as_deref() == Some("true");
    let sort = query_parameters.sort.as_deref().unwrap_or("price");
    let direction = query_parameters.direction.as_deref().unwrap_or("desc");

    let db_access = db_pool.get_ref();

    match get_tokens_data(
        db_access,
        &contract_address,
        page,
        items_per_page,
        buy_now,
        sort,
        direction,
    )
    .await
    {
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("data not found"),
        Ok((collection_data, has_next_page)) => HttpResponse::Ok().json(json!({
            "data": collection_data,
            "has_next_page": has_next_page,
            "next_page": if has_next_page { Some(page + 1) } else { None }
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use crate::db::db_access::MockDb;
    use crate::handlers::collection_handler::get_collection;
    use crate::models::collection::CollectionData;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_get_tokens_handler() {
        let app = test::init_service(App::new().app_data(web::Data::new(MockDb)).route(
            "/collection/0x/tokens",
            web::get().to(get_collection::<MockDb>),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/collection/0x/tokens")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        assert_eq!(status, http::StatusCode::OK);
        let response_body = test::read_body(resp).await;
        let tokens_data: Vec<CollectionData> = serde_json::from_slice(&response_body).unwrap();

        assert_eq!(tokens_data[0].id, Some("1".to_string()));
        assert_eq!(tokens_data[0].address, Some(4));
    }
}

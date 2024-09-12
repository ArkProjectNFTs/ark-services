use ark_marketplace_api::handlers::collection_handler;
use ark_marketplace_api::handlers::default_handler;
use ark_marketplace_api::models::collection::{
    CollectionActivityData, CollectionData, CollectionPortfolioData, CollectionSearchData,
    OwnerData,
};
use ark_marketplace_api::types::collection::{
    AttributeValues, AttributesResponse, CollectionActivityResponse, CollectionPortfolioResponse,
    CollectionResponse, CollectionSearchResponse, CollectionsResponse,
};
use ark_marketplace_api::types::default::{HealthCheckResponse, HealthCheckResponseV1};
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

#[derive(OpenApi)]
#[openapi(
    paths(
        default_handler::root,
        default_handler::health_check,
        collection_handler::get_collection,
        collection_handler::get_collection_activity,
        collection_handler::get_portfolio_collections,
        collection_handler::search_collections,
        collection_handler::get_traits,
        collection_handler::get_collections,
    ),
    components(schemas(
        HealthCheckResponse,
        CollectionResponse,
        CollectionData,
        CollectionActivityResponse,
        CollectionActivityData,
        CollectionPortfolioData,
        CollectionPortfolioResponse,
        CollectionSearchData,
        OwnerData,
        CollectionSearchResponse,
        AttributesResponse,
        AttributeValues,
        CollectionsResponse
    ))
)]
pub struct ApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(default_handler::health_check_v1),
    components(schemas(HealthCheckResponseV1))
)]
pub struct ApiDocV1;

pub fn configure_docs() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
        (
            Url::new("apiv0", "/api-docs/openapi.json"),
            ApiDoc::openapi(),
        ),
        (
            Url::with_primary("apiv1", "/api-docs/openapi_v1.json", true),
            ApiDocV1::openapi(),
        ),
    ])
}

use ark_marketplace_api::handlers::token_handler::RefreshMetadataRequest;
use ark_marketplace_api::handlers::{
    collection_handler, default_handler, portfolio_handler, token_handler,
};
use ark_marketplace_api::models::collection::{
    CollectionActivityData, CollectionData, CollectionFullData, CollectionPortfolioData,
    CollectionSearchData, OwnerData,
};
use ark_marketplace_api::models::portfolio::{OfferApiData, StatsData};
use ark_marketplace_api::models::token::{
    Listing, TokenActivityData, TokenData, TokenDataListing, TokenEventType, TokenInformationData,
    TokenMarketData, TokenOfferOneData, TokenPortfolioActivityData, TokenPortfolioData, TopOffer,
};
use ark_marketplace_api::types::collection::{
    AttributeValues, AttributesResponse, CollectionActivityResponse, CollectionPortfolioResponse,
    CollectionResponse, CollectionSearchResponse, CollectionsResponse,
};
use ark_marketplace_api::types::default::{HealthCheckResponse, HealthCheckResponseV1};
use ark_marketplace_api::types::portfolio::{
    PortfolioActivityResponse, PortfolioOffersResponse, PortfolioStatsResponse,
    TokensPortfolioResponse,
};
use ark_marketplace_api::types::token::{
    TokenActivitiesResponse, TokenMarketDataResponse, TokenOffersResponse, TokenResponse,
    TokensResponse,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

#[derive(OpenApi)]
#[openapi(
    paths(
        default_handler::health_check,
        collection_handler::get_collection,
        collection_handler::get_collection_activity,
        collection_handler::get_portfolio_collections,
        collection_handler::search_collections,
        collection_handler::get_traits,
        collection_handler::get_collections,
        token_handler::get_tokens,
        token_handler::get_token,
        token_handler::get_token_market,
        token_handler::get_token_offers,
        token_handler::get_tokens_portfolio,
        token_handler::get_token_activity,
        token_handler::post_refresh_token_metadata,
        portfolio_handler::get_activity,
        portfolio_handler::get_offers,
        portfolio_handler::get_stats,
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
        CollectionsResponse,
        TokensResponse,
        TokenResponse,
        TokenInformationData,
        TokenData,
        TokenDataListing,
        TokenMarketData,
        TokenMarketDataResponse,
        TokenEventType,
        Listing,
        TopOffer,
        TokenOffersResponse,
        TokenOfferOneData,
        TokenPortfolioData,
        TokensPortfolioResponse,
        TokenActivityData,
        TokenActivitiesResponse,
        OfferApiData,
        TokenPortfolioActivityData,
        PortfolioActivityResponse,
        PortfolioOffersResponse,
        RefreshMetadataRequest,
        PortfolioStatsResponse,
        StatsData,
        CollectionFullData
    ))
)]
pub struct ApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(default_handler::health_check_v1),
    components(schemas(HealthCheckResponseV1))
)]
pub struct _ApiDocV1;

pub fn configure() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
        // later usage
        /*(
            Url::new("apiv1", "/api-docs/openapi_v1.json"),
            ApiDocV1::openapi(),
        ),*/
        (
            Url::with_primary("apiv0", "/api-docs/openapi.json", true),
            ApiDoc::openapi(),
        ),
    ])
}

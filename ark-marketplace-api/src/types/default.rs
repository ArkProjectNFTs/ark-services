use utoipa::ToSchema;
use serde::Serialize;


#[derive(ToSchema, Serialize)]
pub struct HealthCheckResponseV1 {
    #[schema()]
    pub status_v1: String,
}

#[derive(ToSchema, Serialize)]
pub struct HealthCheckResponse {
    #[schema()]
    pub status: String,
}

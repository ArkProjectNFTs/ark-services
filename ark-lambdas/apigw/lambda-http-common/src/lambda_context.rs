//! Initializes the context for ArkStack.

use ark_dynamodb::providers::{metrics::LambdaUsageData, LambdaUsageProvider};
use ark_dynamodb::{init_aws_dynamo_client, pagination::DynamoDbPaginator, DynamoDbCtx};
use lambda_http::{http::StatusCode, Request, RequestExt};
use std::collections::HashMap;
use std::time::Instant;

use crate::{params, HttpParamSource};

use crate::{LambdaHttpError, LambdaHttpResponse};

/// A common context for every http lambda.
#[derive(Debug)]
pub struct LambdaCtx {
    pub table_name: String,
    pub max_items_limit: Option<i32>,
    pub paginator: DynamoDbPaginator,
    pub db: DynamoDbCtx,
    pub api_key: String,
    pub req_id: String,
    pub function_name: String,
    creation_instant: Instant,
}

impl LambdaCtx {
    /// Initializes a lambda context from the given event.
    /// The context is expecting the following fields from the event:
    ///
    /// 1. Stage variables:
    ///    * `tableName` -> name of the dynamodb table.
    ///    * `paginationCache` -> redis URL for pagination cache.
    ///    * `maxItemsLimit` -> the maximum limit of items returned by dynamodb. The hard limit hard coded is 250.
    ///
    /// 2. Headers:
    ///    * `Authorization` -> API key as Authorization bearer OR `x-api-key`?
    ///
    /// 3. Query String params:
    ///    * `cursor` -> the cursor to be used (optional).
    #[allow(clippy::redundant_closure)]
    pub async fn from_event(event: &Request) -> Result<Self, LambdaHttpError> {
        let creation_instant = Instant::now();

        let stage_vars = event.stage_variables();
        let table_name = &stage_vars
            .first("tableName")
            .expect("tableName must be set in stage variables");
        let pagination_db = &stage_vars
            .first("paginationCache")
            .expect("paginationCache must be set in stage variables");

        let max_items_limit = stage_vars
            .first("maxItemsLimit")
            .as_ref()
            .map(|v| v.parse::<i32>().expect("Invalid i32 for max items"));

        let paginator = DynamoDbPaginator::new(pagination_db);

        let api_key = if let Some(apix_header) = event.headers().get("x-api-key") {
            apix_header.to_str().unwrap().to_string()
        } else {
            "NO_APIKEY".to_string()
        };

        let lctx = event.lambda_context();
        let req_id = lctx.request_id;
        let function_name = lctx.env_config.function_name.clone();

        let maybe_cursor = params::string_param(event, "cursor", HttpParamSource::QueryString);

        let last_evaluated_key = if let Some(c) = maybe_cursor {
            paginator
                .get_cursor(&c)
                .map_err(|e| LambdaHttpError::Provider(e))?
        } else {
            None
        };

        let client = init_aws_dynamo_client().await;

        let db = DynamoDbCtx {
            client,
            exclusive_start_key: last_evaluated_key,
        };

        Ok(Self {
            paginator,
            db,
            table_name: table_name.to_string(),
            max_items_limit,
            api_key,
            req_id,
            function_name,
            creation_instant,
        })
    }

    pub async fn register_usage(
        &self,
        params: HashMap<String, String>,
        lambda_response: Option<&LambdaHttpResponse>,
    ) -> Result<(), LambdaHttpError> {
        let exec_time = self.creation_instant.elapsed().as_millis();

        let (status, capacity) = if let Some(lr) = lambda_response {
            (lr.inner.status(), lr.capacity)
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, 0.0)
        };

        let response_status = status.as_u16() as i32;

        let data = LambdaUsageData {
            request_id: self.req_id.clone(),
            api_key: self.api_key.clone(),
            lambda_name: self.function_name.clone(),
            capacity,
            exec_time,
            response_status,
            params,
        };

        LambdaUsageProvider::register_usage(
            &self.db.client,
            &format!("{}_lambda_usage", self.table_name),
            &data,
        )
        .await
        .map_err(LambdaHttpError::Provider)?;

        Ok(())
    }
}

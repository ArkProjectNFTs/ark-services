//! Initializes the context for ArkStack.

use ark_dynamodb::{init_aws_dynamo_client, pagination::DynamoDbPaginator, DynamoDbCtx};
use ark_sqlx::providers::metrics::{LambdaUsageData, LambdaUsageProvider};
use ark_sqlx::providers::SqlxCtx;
use lambda_http::{http::StatusCode, request::RequestContext, Request, RequestExt};
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
    pub dynamodb: DynamoDbCtx,
    pub sqlx: SqlxCtx,
    pub api_key: String,
    pub req_id: String,
    pub function_name: String,
    pub stage_name: String,
    // TODO: maybe almost everything can be private.
    http_method: String,
    http_path: String,
    source_ip: String,
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
            .first("lambdaUsageTable")
            .expect("lambdaUsageTable must be set in stage variables");
        let stage_name = &stage_vars
            .first("stageName")
            .expect("stageName must be set in stage variables");
        let sqlx_url = &stage_vars
            .first("sqlxUrl")
            .expect("sqlxUrl must be set in stage variables");
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

        let dynamodb = DynamoDbCtx {
            client,
            exclusive_start_key: last_evaluated_key,
        };

        let sqlx = SqlxCtx::new(sqlx_url).await?;

        let (http_method, http_path, source_ip) = http_info_from_context(&event.request_context());

        Ok(Self {
            paginator,
            dynamodb,
            sqlx,
            table_name: table_name.to_string(),
            max_items_limit,
            api_key,
            req_id,
            function_name,
            creation_instant,
            http_method,
            http_path,
            source_ip,
            stage_name: stage_name.to_string(),
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
            http_method: self.http_method.clone(),
            http_path: self.http_path.clone(),
            source_ip: self.source_ip.clone(),
            stage_name: self.stage_name.clone(),
            capacity,
            exec_time,
            response_status,
            params,
        };

        LambdaUsageProvider::register_usage(
            &self.sqlx,
            &format!("{}_lambda_usage", self.table_name),
            &data,
        )
        .await
        .map_err(LambdaHttpError::SqlxProvider)?;

        Ok(())
    }
}

fn http_info_from_context(context: &RequestContext) -> (String, String, String) {
    match context {
        RequestContext::ApiGatewayV1(c) => {
            let method = c.http_method.as_str().to_string();
            let source_ip = if let Some(ip) = &c.identity.source_ip {
                ip.to_string()
            } else {
                String::new()
            };

            let path = if let Some(p) = &c.path {
                p.to_string()
            } else {
                String::new()
            };

            (method, path, source_ip)
        }
        RequestContext::ApiGatewayV2(c) => {
            let method = c.http.method.as_str().to_string();
            let source_ip = if let Some(ip) = &c.http.source_ip {
                ip.to_string()
            } else {
                String::new()
            };

            let path = if let Some(p) = &c.http.path {
                p.to_string()
            } else {
                String::new()
            };

            (method, path, source_ip)
        }
        _ => (String::new(), String::new(), String::new()),
    }
}

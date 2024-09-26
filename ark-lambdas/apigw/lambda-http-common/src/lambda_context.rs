//! Initializes the context for ArkStack.

use crate::LambdaHttpError;
use crate::{params, HttpParamSource};
use ark_dynamodb::{init_aws_dynamo_client, pagination::DynamoDbPaginator, DynamoDbCtx};
use lambda_http::{Request, RequestExt};
use std::collections::HashMap;

/// A common context for every http lambda.
#[derive(Debug)]
pub struct LambdaCtx {
    pub table_name: String,
    pub usage_table_name: String,
    pub max_items_limit: Option<i32>,
    pub paginator: DynamoDbPaginator,
    pub dynamodb: DynamoDbCtx,
    pub api_key: String,
    pub req_id: String,
    pub function_name: String,
    pub stage_name: String,
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
        let stage_vars = event.stage_variables();

        let table_name = &stage_vars
            .first("tableName")
            .expect("tableName must be set in stage variables");

        let usage_table_name = &stage_vars
            .first("lambdaUsageTable")
            .expect("lambdaUsageTable must be set in stage variables");

        let stage_name = &stage_vars
            .first("stageName")
            .expect("stageName must be set in stage variables");

        // let sqlx_url = &stage_vars
        //     .first("sqlxUrl")
        //     .expect("sqlxUrl must be set in stage variables");

        let pagination_db = &stage_vars
            .first("paginationCache")
            .expect("paginationCache must be set in stage variables");

        let max_items_limit_default = stage_vars
            .first("maxItemsLimit")
            .as_ref()
            .map(|v| v.parse::<i32>().expect("Invalid i32 for max items"));

        let limit_param = params::string_param(event, "limit", HttpParamSource::QueryString)
            .and_then(|limit_str| limit_str.parse::<i32>().ok());

        let max_items_limit = limit_param.or(max_items_limit_default);

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

        let (lek_single, lek_multiple) = if let Some(c) = &maybe_cursor {
            let lek = paginator
                .get_cursor(c)
                .map_err(|e| LambdaHttpError::Provider(e))?;

            let leks = if lek.is_none() {
                paginator
                    .get_cursor_multiple(c)
                    .map_err(|e| LambdaHttpError::Provider(e))?
            } else {
                HashMap::new()
            };

            (lek, leks)
        } else {
            (None, HashMap::new())
        };

        let client = init_aws_dynamo_client().await;

        let dynamodb = DynamoDbCtx {
            client,
            exclusive_start_key: lek_single,
            multiple_exclusive_start_keys: lek_multiple,
        };

        Ok(Self {
            paginator,
            dynamodb,
            table_name: table_name.to_string(),
            usage_table_name: usage_table_name.to_string(),
            max_items_limit,
            api_key,
            req_id,
            function_name,
            stage_name: stage_name.to_string(),
        })
    }
}

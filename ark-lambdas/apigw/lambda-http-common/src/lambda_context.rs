//! Initializes the context for ArkStack.

use ark_dynamodb::{init_aws_dynamo_client, pagination::DynamoDbPaginator, DynamoDbCtx};
use lambda_http::{Request, RequestExt};

use crate::{params, HttpParamSource};

use crate::LambdaHttpError;

/// A common context for every http lambda.
#[derive(Debug)]
pub struct LambdaCtx {
    pub table_name: String,
    pub paginator: DynamoDbPaginator,
    pub db: DynamoDbCtx,
}

impl LambdaCtx {
    /// Initializes a lambda context from the given event.
    /// The context is expecting the following fields from the event:
    ///
    /// 1. Stage variables:
    ///    * `tableName` -> name of the dynamodb table.
    ///    * `paginationCache` -> redis URL for pagination cache.
    ///
    /// 2. Headers:
    ///    * `Authorization` -> API key as Authorization bearer.
    ///
    /// 3. Query String params:
    ///    * `cursor` -> the cursor to be used (optional).
    pub async fn from_event(event: &Request) -> Result<Self, LambdaHttpError> {
        let stage_vars = event.stage_variables();
        let table_name = &stage_vars
            .first("tableName")
            .expect("tableName must be set in stage variables");
        let pagination_db = &stage_vars
            .first("paginationCache")
            .expect("paginationCache must be set in stage variables");

        let paginator = DynamoDbPaginator::new(pagination_db);

        let maybe_cursor = params::string_param(&event, "cursor", HttpParamSource::QueryString);

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
        })
    }
}

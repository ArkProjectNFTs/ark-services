use ark_dynamodb::providers::{ArkTokenProvider, DynamoDbTokenProvider};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{
    self as common, ArkApiResponse, HttpParamSource, LambdaCtx, LambdaHttpError,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let ctx = LambdaCtx::from_event(&event).await?;
    let provider = DynamoDbTokenProvider::new(&ctx.table_name, None);
    let (address, token_id_hex) = get_params(&event)?;

    match provider
        .get_last_refresh_token_metadata(&ctx.db, address.as_str(), token_id_hex.as_str())
        .await
    {
        Ok(last_refresh_timestamp_option) => {
            if let Some(last_refresh_timestamp) = last_refresh_timestamp_option {
                // Calculate the current timestamp
                let current_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as i64;

                // Check if last_refresh_timestamp is greater than 10 minutes ago
                if current_timestamp - last_refresh_timestamp < 10 * 60 {
                    error!(
                        "Attempt to refresh token metadata for token {} of contract {} too soon.",
                        token_id_hex, address
                    );
                    return common::bad_request_rsp(
                        "Metadata refresh can only be performed every 10 minutes.",
                    );
                }
            }

            // If more than 10 minutes have passed, proceed to update the token metadata status
            match provider
                .update_token_metadata_status(
                    &ctx.db,
                    address.as_str(),
                    token_id_hex.as_str(),
                    "true",
                )
                .await
            {
                Ok(_) => {
                    info!(
                        "Successfully updated token metadata status for token {} of contract {}",
                        token_id_hex, address
                    );
                    common::ok_body_rsp(&ArkApiResponse {
                        cursor: None,
                        result: "We've queued this token to update its metadata! It will be updated soon.",
                    })
                }
                Err(e) => {
                    error!(
                        "Failed to update token metadata status for token {} of contract {}: {}",
                        token_id_hex, address, e
                    );
                    common::internal_server_error_rsp("Failed to refresh token metadata")
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to get last refresh timestamp for token {} of contract {}: {}",
                token_id_hex, address, e
            );
            common::internal_server_error_rsp("Failed to get token metadata last refresh timestamp")
        }
    }
}

fn get_params(event: &Request) -> Result<(String, String), LambdaHttpError> {
    let address = match common::require_hex_param(event, "contract_address", HttpParamSource::Path)
    {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    let token_id_hex =
        match common::require_hex_or_dec_param(event, "token_id", HttpParamSource::Path) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

    Ok((address, token_id_hex))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(|event: Request| async {
        function_handler(event).await
    }))
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::RequestExt;
    use lambda_http_common::format::pad_hex;
    use std::collections::HashMap;

    #[tokio::test]
    async fn params_ok() {
        let address = "0x1234".to_string();
        let token_id = "1".to_string();

        let mut params = HashMap::new();
        params.insert("contract_address".to_string(), address.clone());
        params.insert("token_id".to_string(), token_id.clone());

        let req = Request::default().with_path_parameters(params.clone());

        let (address, token_id_hex) = get_params(&req).unwrap();

        assert_eq!(address, pad_hex(&address));
        assert_eq!(token_id_hex, pad_hex(&token_id));
    }
}

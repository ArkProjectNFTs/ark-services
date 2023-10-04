use async_trait::async_trait;
use aws_sdk_dynamodb::Client as DynamoClient;

use super::{types::TokenData, ArkTokenProvider};
use crate::ProviderError;

/// DynamoDB provider for tokens.
pub struct DynamoDbTokenProvider;

impl Default for DynamoDbTokenProvider {
    fn default() -> Self {
        DynamoDbTokenProvider
    }
}

#[async_trait]
impl ArkTokenProvider for DynamoDbTokenProvider {
    type Client = DynamoClient;

    async fn get_token(
        &self,
        _client: &Self::Client,
        _address: &str,
    ) -> Result<TokenData, ProviderError> {
        // TODO: call dynamo.

        Ok(TokenData {
            block_number: 123,
            mint_timestamp: 8888,
            mint_address: "0x1234".to_string(),
            owner: "0x222".to_string(),
            token_id: "0x1".to_string(),
            contract_address: "0x2234".to_string(),
        })
    }
}

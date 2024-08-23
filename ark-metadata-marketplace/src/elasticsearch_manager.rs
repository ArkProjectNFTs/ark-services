use arkproject::metadata::{
    elasticsearch_manager::ElasticsearchManager,
    types::{RequestError, TokenMetadata},
};
use async_trait::async_trait;
use reqwest::Client as ReqwestClient;
use serde_json::json;

pub struct EsManager {
    client: ReqwestClient,
    elasticsearch_url: String,
    username: String,
    password: String,
}

#[async_trait]
impl ElasticsearchManager for EsManager {
    async fn upsert_token_metadata(
        &self,
        contract_address: &str,
        token_id: &str,
        chain_id: &str,
        metadata: TokenMetadata,
    ) -> Result<(), RequestError> {
        let doc_id = format!("{}_{}_{}", contract_address, chain_id, token_id);
        let url = format!("{}/nft-metadata/_update/{}", self.elasticsearch_url, doc_id);

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&json!({
                "doc": {
                    "contract_address": contract_address,
                    "token_id": token_id,
                    "chain_id": chain_id,
                    "metadata": metadata.normalized,
                    "raw_metadata": metadata.raw,
                    "metadata_updated_at": metadata.metadata_updated_at
                },
                "doc_as_upsert": true
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(RequestError::Reqwest(error_message))
        }
    }
}

impl EsManager {
    pub fn new(elasticsearch_url: String, username: String, password: String) -> Self {
        Self {
            client: ReqwestClient::new(),
            elasticsearch_url,
            username,
            password,
        }
    }
}

#[async_trait]
impl ElasticsearchManager for NoOpElasticsearchManager {
    async fn upsert_token_metadata(
        &self,
        _contract_address: &str,
        _token_id: &str,
        _chain_id: &str,
        _metadata: TokenMetadata,
    ) -> Result<(), RequestError> {
        Ok(())
    }
}

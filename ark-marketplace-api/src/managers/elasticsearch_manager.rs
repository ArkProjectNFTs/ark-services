use reqwest::Client as ReqwestClient;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ElasticsearchManager {
    client: ReqwestClient,
    es_data: HashMap<String, String>,
}

impl ElasticsearchManager {
    pub fn new(es_data: HashMap<String, String>) -> Self {
        Self {
            client: ReqwestClient::new(),
            es_data
        }
    }

    pub async fn get_attributes_for_collection(
        &self,
        collection_id: &str,
        chain_id: &str,
    ) -> Result<HashMap<String, HashMap<String, usize>>, Box<dyn std::error::Error>> {
        let url = format!("{}/nft-metadata/_search", self.get_es_url());

        let body = json!({
            "_source": ["metadata.attributes.trait_type", "metadata.attributes.value"],
            "query": {
                "bool": {
                    "must": [
                        {
                            "term": {
                                "contract_address.keyword": collection_id
                            }
                        },
                        {
                            "term": {
                                "chain_id.keyword": chain_id
                            }
                        }
                    ]
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .basic_auth(self.get_username(), Some(self.get_password()))
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let json_response: Value = response.json().await?;
            let traits_map = Self::process_elasticsearch_response(&json_response);
            Ok(traits_map)
        } else {
            Err(format!("Request failed with status: {}", response.status()).into())
        }
    }

    /// Parse response
    fn process_elasticsearch_response(response: &Value) -> HashMap<String, HashMap<String, usize>> {
        let mut traits_map: HashMap<String, HashMap<String, usize>> = HashMap::new();

        if let Some(hits) = response.get("hits").and_then(|h| h.get("hits")) {
            if let Some(hits_array) = hits.as_array() {
                for hit in hits_array {
                    if let Some(attributes) = hit
                        .get("_source")
                        .and_then(|s| s.get("metadata"))
                        .and_then(|m| m.get("attributes"))
                        .and_then(|a| a.as_array())
                    {
                        for attribute in attributes {
                            if let Some(trait_type) =
                                attribute.get("trait_type").and_then(|t| t.as_str())
                            {
                                if let Some(value) = attribute.get("value").and_then(|v| v.as_str())
                                {
                                    let entry =
                                        traits_map.entry(trait_type.to_string()).or_default();
                                    *entry.entry(value.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        traits_map
    }

    fn get_es_url(&self) -> &str {
        self.es_data.get("url").map(String::as_str).unwrap_or("URL not found")
    }

    fn get_username(&self) -> &str {
        self.es_data.get("username").map(String::as_str).unwrap_or("Username not found")
    }

    fn get_password(&self) -> &str {
        self.es_data.get("password").map(String::as_str).unwrap_or("Password not found")
    }
}

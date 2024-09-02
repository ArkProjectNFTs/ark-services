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
            es_data,
        }
    }

    pub async fn get_attributes_for_collection(
        &self,
        collection_id: &str,
        chain_id: &str,
    ) -> Result<HashMap<String, HashMap<String, usize>>, Box<dyn std::error::Error>> {
        let url = format!("{}/nft-metadata/_search?scroll=1m", self.get_es_url());
        let scroll_url = format!("{}/_search/scroll", self.get_es_url());

        let body = json!({
            "_source": ["metadata.attributes.trait_type", "metadata.attributes.value"],
            "size": 10000,
            "query": {
                "bool": {
                    "must": [
                        {
                            "term": {
                                "contract_address": collection_id
                            }
                        },
                        {
                            "term": {
                                "chain_id": chain_id
                            }
                        }
                    ]
                }
            }
        });

        let mut traits_map: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut scroll_id: Option<String> = None;

        loop {
            let response = if let Some(ref id) = scroll_id {
                // Request with the scroll_id to get the next batch
                self.client
                    .post(&scroll_url)
                    .basic_auth(self.get_username(), Some(self.get_password()))
                    .json(&json!({ "scroll": "1m", "scroll_id": id }))
                    .send()
                    .await?
            } else {
                // Initial search request
                self.client
                    .post(&url)
                    .basic_auth(self.get_username(), Some(self.get_password()))
                    .json(&body)
                    .send()
                    .await?
            };

            if response.status().is_success() {
                let json_response: Value = response.json().await?;
                // Update traits_map with the new batch of results
                let new_traits_map = Self::process_elasticsearch_response(&json_response);
                for (key, value_map) in new_traits_map {
                    let entry = traits_map.entry(key).or_insert_with(HashMap::new);
                    for (value_key, count) in value_map {
                        *entry.entry(value_key).or_insert(0) += count;
                    }
                }

                // Check if there are more results to scroll
                let empty_vec = vec![];
                let hits = json_response["hits"]["hits"]
                    .as_array()
                    .unwrap_or(&empty_vec);
                if hits.is_empty() {
                    break;
                }
                if hits.is_empty() {
                    break;
                }

                // Update scroll_id for the next request
                scroll_id = json_response["_scroll_id"].as_str().map(String::from);
            } else {
                return Err(format!("Request failed with status: {}", response.status()).into());
            }
        }

        Ok(traits_map)
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

    pub async fn search_tokens_by_traits(
        &self,
        collection_id: &str,
        chain_id: &str,
        traits: HashMap<String, Vec<String>>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("{}/nft-metadata/_search?scroll=1m", self.get_es_url());
        let scroll_url = format!("{}/_search/scroll", self.get_es_url());

        let mut must_clauses = Vec::new();

        for (trait_type, values) in traits.iter() {
            let mut should_clauses = Vec::new();

            for value in values.iter() {
                should_clauses.push(json!({
                    "term": {
                        "metadata.attributes.value": value
                    }
                }));
            }

            must_clauses.push(json!({
                "nested": {
                    "path": "metadata.attributes",
                    "query": {
                        "bool": {
                            "must": [
                                { "term": { "metadata.attributes.trait_type": trait_type } },
                                {
                                    "bool": {
                                        "should": should_clauses
                                    }
                                }
                            ]
                        }
                    }
                }
            }));
        }

        must_clauses.push(json!({
            "term": {
                "contract_address": collection_id
            }
        }));

        must_clauses.push(json!({
            "term": {
                "chain_id": chain_id
            }
        }));

        let body = json!({
            "_source": ["token_id"],
            "size": 10000,
            "query": {
                "bool": {
                    "must": must_clauses
                }
            }
        });

        let mut token_ids = Vec::new();
        let mut scroll_id: Option<String> = None;

        loop {
            let response = if let Some(ref id) = scroll_id {
                // Request with the scroll_id to get the next batch
                self.client
                    .post(&scroll_url)
                    .basic_auth(self.get_username(), Some(self.get_password()))
                    .json(&json!({ "scroll": "1m", "scroll_id": id }))
                    .send()
                    .await?
            } else {
                // Initial search request
                self.client
                    .post(&url)
                    .basic_auth(self.get_username(), Some(self.get_password()))
                    .json(&body)
                    .send()
                    .await?
            };

            if response.status().is_success() {
                let json_response: Value = response.json().await?;
                let new_token_ids = Self::extract_token_ids(&json_response);
                token_ids.extend(new_token_ids);

                // Check if there are more results to scroll
                let empty_vec = Vec::new();
                let hits = json_response["hits"]["hits"]
                    .as_array()
                    .unwrap_or(&empty_vec);
                if hits.is_empty() {
                    break;
                }
                if hits.is_empty() {
                    break;
                }

                // Update scroll_id for the next request
                scroll_id = json_response["_scroll_id"].as_str().map(String::from);
            } else {
                return Err(format!("Request failed with status: {}", response.status()).into());
            }
        }

        Ok(token_ids)
    }

    fn extract_token_ids(response: &Value) -> Vec<String> {
        let mut token_ids = Vec::new();

        if let Some(hits) = response.get("hits").and_then(|h| h.get("hits")) {
            if let Some(hits_array) = hits.as_array() {
                for hit in hits_array {
                    if let Some(token_id) = hit
                        .get("_source")
                        .and_then(|s| s.get("token_id"))
                        .and_then(|t| t.as_str())
                    {
                        token_ids.push(token_id.to_string());
                    }
                }
            }
        }

        token_ids
    }

    fn get_es_url(&self) -> &str {
        self.es_data
            .get("url")
            .map(String::as_str)
            .unwrap_or("URL not found")
    }

    fn get_username(&self) -> &str {
        self.es_data
            .get("username")
            .map(String::as_str)
            .unwrap_or("Username not found")
    }

    fn get_password(&self) -> &str {
        self.es_data
            .get("password")
            .map(String::as_str)
            .unwrap_or("Password not found")
    }
}

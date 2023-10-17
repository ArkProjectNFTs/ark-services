use aws_sdk_dynamodb::types::AttributeValue;
use redis::Client as RedisClient;
use redis::Commands;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::ProviderError;

/// A paginator for DynamoDB operations.
/// The pagination is made using the `last_evaluated_key`
/// optionally returned by a DynamoDB operation.
///
/// By using REDIS in memory caching, the paginator
/// is able to recover the `last_evaluated_key` from
/// the cache to then serve subsequent requests.
///
/// A TTL (time-to-live) is set to avoid accumulating too
/// much pagination records, currently the ttl is 1h.
#[derive(Debug)]
pub struct DynamoDbPaginator {
    client: RedisClient,
}

impl DynamoDbPaginator {
    /// Instanciates a new paginator with underlying
    /// cache client.
    pub fn new(redis_url: &str) -> Self {
        let client =
            RedisClient::open(redis_url).expect("Can't initialize redis connection for pagination");

        Self { client }
    }

    /// Get the cursor (`last_evaluated_key`) for the given
    /// `hash_key`. The `hash_key` is obtained from `store_cursor` function.
    pub fn get_cursor(
        &self,
        hash_key: &str,
    ) -> Result<Option<HashMap<String, AttributeValue>>, ProviderError> {
        if let Ok(mut conn) = self.client.get_connection() {
            let data: Option<HashMap<String, String>> = conn
                .hgetall(hash_key)
                .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;

            if let Some(d) = data {
                let mut map: HashMap<String, AttributeValue> = HashMap::new();
                for (k, v) in d {
                    map.insert(k.clone(), AttributeValue::S(v.clone()));
                }

                Ok(Some(map))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Stores the given `last_evaluated_key` content in cache.
    /// A `hash_key` is associated with the given value for it's
    /// retrieval. Returns `None` if the cursor is not existing.
    pub fn store_cursor(
        &self,
        last_evaluated_key: &Option<HashMap<String, AttributeValue>>,
    ) -> Result<Option<String>, ProviderError> {
        if let Some(lek) = last_evaluated_key {
            if let Ok(mut conn) = self.client.get_connection() {
                let hash_key: String = Uuid::new_v4().to_hyphenated().to_string();

                for (key, value) in lek {
                    let value = value
                        .as_s()
                        .expect("Paginator service only support String keys in LEK");
                    conn.hset(hash_key.clone(), key, value)
                        .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;
                }

                let ttl = get_hash_ttl() as usize;
                conn.expire(hash_key.clone(), ttl)
                    .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;

                Ok(Some(hash_key))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

fn get_hash_ttl() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting time");

    // 1h ttl, check if this is enough.
    (now + Duration::from_secs(60 * 60)).as_secs()
}

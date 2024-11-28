use super::ProviderError;
use aws_sdk_dynamodb::types::AttributeValue;
use redis::Client as RedisClient;
use redis::Commands;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, trace};
use uuid::Uuid;

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

pub type Lek = HashMap<String, AttributeValue>;

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
    pub fn get_cursor(&self, hash_key: &str) -> Result<Option<Lek>, ProviderError> {
        if let Ok(mut conn) = self
            .client
            .get_connection_with_timeout(Duration::from_secs(2))
        {
            let data: Option<HashMap<String, String>> = conn
                .hgetall(hash_key)
                .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;

            if let Some(d) = data {
                let mut map: Lek = Lek::new();
                for (k, v) in d {
                    if k == "GSI6SK" {
                        map.insert(k.clone(), AttributeValue::N(v.clone()));
                    } else {
                        map.insert(k.clone(), AttributeValue::S(v.clone()));
                    }
                }
                Ok(Some(map))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get the cursors (`last_evaluated_key`) for the given
    /// `hash_key`. In some situations, several queries are paginated,
    /// and we want to keep a pagination state with all of them synchronized.
    /// In that situation, one `hash_key` will contains a list of `hash_keys`, which
    /// must then be retrieved to have all the cursors synchronized.
    ///
    /// The cursors are stored in a `HashMap`, named given by the user.
    pub fn get_cursor_multiple(
        &self,
        hash_key: &str,
    ) -> Result<HashMap<String, Option<Lek>>, ProviderError> {
        let mut cursors: HashMap<String, Option<Lek>> = HashMap::new();

        if let Ok(mut conn) = self
            .client
            .get_connection_with_timeout(Duration::from_secs(2))
        {
            // Get all the keys for the multiple cursors.
            let keys: Option<HashMap<String, String>> = conn
                .hgetall(hash_key)
                .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;

            if let Some(keys) = keys {
                for (cursor_name, cursor_hash_key) in keys {
                    cursors.insert(cursor_name, self.get_cursor(&cursor_hash_key)?);
                }
            }
        }

        Ok(cursors)
    }

    /// Stores the given `last_evaluated_key` content in cache.
    /// A `hash_key` is associated with the given value for it's
    /// retrieval. Returns `None` if the cursor is not existing.
    pub fn store_cursor(
        &self,
        last_evaluated_key: &Option<Lek>,
    ) -> Result<Option<String>, ProviderError> {
        trace!("Storing cursor: {:?}", last_evaluated_key);

        let lek = match last_evaluated_key {
            Some(lek) => lek,
            None => {
                debug!("'last_evaluated_key' is required but was not provided.");
                return Ok(None);
            }
        };

        let mut conn = match self
            .client
            .get_connection_with_timeout(Duration::from_secs(2))
        {
            Ok(conn) => conn,
            Err(_) => {
                error!("Failed to establish a connection with the paginator client.");
                return Ok(None);
            }
        };

        let hash_key: String = Uuid::new_v4().to_hyphenated().to_string();

        for (key, value) in lek {
            let result = match value {
                AttributeValue::S(s) => s,
                AttributeValue::N(n) => n,
                _ => {
                    return Err(ProviderError::PaginationCacheError(
                        "LEK parsing error (unknown type)".to_string(),
                    ));
                }
            };

            let _: () = conn
                .hset(&hash_key, key, result)
                .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;
        }

        let ttl = get_hash_ttl();
        let _: () = conn
            .expire(&hash_key, ttl)
            .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;

        Ok(Some(hash_key))
    }

    /// Stores the given `last_evaluated_keys` content in cache.
    /// A `hash_key` is associated with a `HashMap` that contains
    /// all the cursors `hash_key`s.
    pub fn store_cursor_multiple(
        &self,
        last_evaluated_keys: &HashMap<String, Option<Lek>>,
    ) -> Result<Option<String>, ProviderError> {
        if let Ok(mut conn) = self
            .client
            .get_connection_with_timeout(Duration::from_secs(2))
        {
            let hash_key: String = Uuid::new_v4().to_hyphenated().to_string();

            for (lek_name, lek) in last_evaluated_keys {
                if let Some(lek_key) = self.store_cursor(lek)? {
                    let _: () = conn
                        .hset(hash_key.clone(), lek_name, lek_key)
                        .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;
                }
            }

            let ttl = get_hash_ttl();
            let _: () = conn
                .expire(hash_key.clone(), ttl)
                .map_err(|e| ProviderError::PaginationCacheError(e.to_string()))?;

            Ok(Some(hash_key))
        } else {
            Ok(None)
        }
    }
}

fn get_hash_ttl() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error getting time");

    // 1h ttl, check if this is enough.
    (now + Duration::from_secs(60 * 60)).as_secs() as i64
}

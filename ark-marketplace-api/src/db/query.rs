use crate::db::db_access::DatabaseAccess;
use crate::models::collection::{CollectionData, CollectionPortfolioData};
use crate::models::token::{TokenData, TokenOneData, TokenPortfolioData};
use redis::AsyncCommands;

pub async fn get_collections_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    page: i64,
    items_per_page: i64,
    time_range: &str,
) -> Result<Vec<CollectionData>, sqlx::Error> {
    db_access
        .get_collections_data(page, items_per_page, time_range, None)
        .await
}

pub async fn get_portfolio_collections_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    user_address: &str,
    page: i64,
    items_per_page: i64,
) -> Result<(Vec<CollectionPortfolioData>, bool, i64), sqlx::Error> {
    db_access
        .get_portfolio_collections_data(page, items_per_page, user_address)
        .await
}

pub async fn get_collection_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    redis_conn: &mut redis::aio::MultiplexedConnection,
    contract_address: &str,
    chain_id: &str,
) -> Result<CollectionData, sqlx::Error> {
    // Generate a unique key for this query based on buy_now value
    let cache_key = format!("collection_{}", contract_address);
    // Try to get the data from Redis
    let cached_data: Option<String> = redis_conn.get(&cache_key).await.unwrap_or(None);

    match cached_data {
        Some(data) => {
            // If the data is in the cache, deserialize it and return it
            match serde_json::from_str::<CollectionData>(&data) {
                Ok(collection_data) => Ok(collection_data),
                Err(e) => {
                    tracing::error!("Failed to deserialize data from Redis: {}", e);
                    Err(sqlx::Error::Configuration(e.into()))
                }
            }
        }
        None => {
            // If the data is not in the cache, get it from the database
            let collection_data = db_access
                .get_collection_data(contract_address, chain_id)
                .await?;

            // Spawn a new task to cache the data in Redis for future requests
            let collection_data_clone = collection_data.clone();
            let cache_key_clone = cache_key.clone();
            let mut redis_conn_clone = redis_conn.clone();
            tokio::spawn(async move {
                let collection_data_string = match serde_json::to_string(&collection_data_clone) {
                    Ok(string) => string,
                    Err(e) => {
                        tracing::error!("Failed to serialize data to Redis: {}", e);
                        return;
                    }
                };
                let _: () = redis_conn_clone
                    .set_ex(&cache_key_clone, collection_data_string, 60 * 60 * 2)
                    .await
                    .unwrap_or(()); // Cache for 2 hours
            });

            Ok(collection_data)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_tokens_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    page: i64,
    items_per_page: i64,
    buy_now: bool,
    sort: &str,
    direction: &str,
) -> Result<(Vec<TokenData>, bool), sqlx::Error> {
    db_access
        .get_tokens_data(
            contract_address,
            chain_id,
            page,
            items_per_page,
            buy_now,
            Some(sort.to_string()),
            Some(direction.to_string()),
        )
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_token_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    token_id: &str,
) -> Result<TokenOneData, sqlx::Error> {
    db_access
        .get_token_data(contract_address, chain_id, token_id)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_tokens_portfolio_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    user_address: &str,
    page: i64,
    items_per_page: i64,
    buy_now: bool,
    sort: &str,
    direction: &str,
    collection: &str,
) -> Result<(Vec<TokenPortfolioData>, bool, i64), sqlx::Error> {
    db_access
        .get_tokens_portfolio_data(
            user_address,
            page,
            items_per_page,
            buy_now,
            sort,
            direction,
            collection,
        )
        .await
}

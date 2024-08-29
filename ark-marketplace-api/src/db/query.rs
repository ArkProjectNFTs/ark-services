use crate::db::db_access::DatabaseAccess;
use crate::utils::http_utils::{
    get_address_from_starknet_id, get_image_from_starknet_address, get_starknet_id_from_address,
};
use regex::Regex;

use crate::models::collection::{
    CollectionActivityData, CollectionData, CollectionFloorPrice, CollectionPortfolioData,
    CollectionSearchData, OwnerDataCompleted,
};
use crate::models::token::{
    TokenActivityData, TokenData, TokenEventType, TokenInformationData, TokenMarketData,
    TokenOfferOneDataDB, TokenPortfolioData,
};
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

pub async fn search_collections_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    query_search: &str,
    items: i64,
) -> Result<(Vec<CollectionSearchData>, Vec<OwnerDataCompleted>), sqlx::Error> {
    let mut cleaned_query_search = query_search.to_string();
    let mut starknet_id: Option<String> = None;
    let mut starknet_address = String::new();
    let mut starknet_image: Option<String> = None;
    // Check if query_search is a starknet.id and get the associated address
    if cleaned_query_search.ends_with(".stark") {
        starknet_id = Some(cleaned_query_search.clone());
        if let Ok(Some(address)) = get_address_from_starknet_id(query_search).await {
            cleaned_query_search = address.clone();
            starknet_address = address;
        }
    } else {
        starknet_address = cleaned_query_search.clone();
        if let Ok(Some(stark_id)) = get_starknet_id_from_address(query_search).await {
            starknet_id = Some(stark_id);
        }
    }

    // get the image if multiple take the first one
    if let Ok(Some(image)) = get_image_from_starknet_address(&starknet_address).await {
        starknet_image = Some(image);
    }

    let re = Regex::new(r"^0x0*").unwrap();
    cleaned_query_search = re.replace(&cleaned_query_search, "").to_string();

    let (collections, accounts) = db_access
        .search_collections_data(Some(&cleaned_query_search), items)
        .await?;

    let completed_accounts = accounts
        .into_iter()
        .map(|account| OwnerDataCompleted {
            owner: account.owner,
            chain_id: account.chain_id,
            starknet_id: starknet_id.clone(),
            image: starknet_image.clone(),
        })
        .collect();

    Ok((collections, completed_accounts))
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
                    .set_ex(&cache_key_clone, collection_data_string, 60)
                    .await
                    .unwrap_or(()); // Cache for 2 hours
            });

            Ok(collection_data)
        }
    }
}

pub async fn get_collection_activity_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    page: i64,
    items_per_page: i64,
    direction: &str,
    types: &Option<Vec<TokenEventType>>,
) -> Result<(Vec<CollectionActivityData>, bool, i64), sqlx::Error> {
    let collection_activity_data = db_access
        .get_collection_activity_data(
            contract_address,
            chain_id,
            page,
            items_per_page,
            direction,
            types,
        )
        .await?;

    Ok(collection_activity_data)
}

pub async fn get_collection_floor_price<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
) -> Result<CollectionFloorPrice, sqlx::Error> {
    db_access
        .get_collection_floor_price(contract_address, chain_id)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_token_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    token_id: &str,
) -> Result<TokenInformationData, sqlx::Error> {
    db_access
        .get_token_data(contract_address, chain_id, token_id)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_tokens_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    redis_conn: &mut redis::aio::MultiplexedConnection,
    contract_address: &str,
    chain_id: &str,
    page: i64,
    items_per_page: i64,
    buy_now: bool,
    sort: &str,
    direction: &str,
    disable_cache: bool,
    token_ids: Option<Vec<String>>,
) -> Result<(Vec<TokenData>, bool, i64), sqlx::Error> {
    // Generate a unique key for this query based on buy_now value
    let cache_key = if buy_now {
        if direction == "asc" {
            format!("listed_tokens_asc_{}_page_{}", contract_address, page)
        } else {
            format!("listed_tokens_desc_{}_page_{}", contract_address, page)
        }
    } else {
        format!("all_tokens_{}_page_{}", contract_address, page)
    };
    // Try to get the data from Redis
    let cached_data: Option<String> = redis_conn.get(&cache_key).await.unwrap_or(None);

    match (cached_data, disable_cache) {
        (Some(data), false) => {
            // If the data is in the cache and caching is not disabled, deserialize it and return it
            match serde_json::from_str::<(Vec<TokenData>, bool, i64)>(&data) {
                Ok(tokens_data) => Ok(tokens_data),
                Err(e) => {
                    tracing::error!("Failed to deserialize data from Redis: {}", e);
                    Err(sqlx::Error::Configuration(e.into()))
                }
            }
        }
        _ => {
            // If the data is not in the cache or caching is disabled, get it from the database
            let tokens_data = db_access
                .get_tokens_data(
                    contract_address,
                    chain_id,
                    page,
                    items_per_page,
                    buy_now,
                    Some(sort.to_string()),
                    Some(direction.to_string()),
                    token_ids,
                )
                .await?;

            // Spawn a new task to cache the data in Redis for future requests
            if !disable_cache {
                let tokens_data_clone = tokens_data.clone();
                let cache_key_clone = cache_key.clone();
                let mut redis_conn_clone = redis_conn.clone();
                tokio::spawn(async move {
                    let tokens_data_string = match serde_json::to_string(&tokens_data_clone) {
                        Ok(string) => string,
                        Err(e) => {
                            tracing::error!("Failed to serialize data to Redis: {}", e);
                            return;
                        }
                    };
                    let _: () = redis_conn_clone
                        .set_ex(&cache_key_clone, tokens_data_string, 60)
                        .await
                        .unwrap_or(()); // Cache for 2 hours
                });
            }

            Ok(tokens_data)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_token_marketdata<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    token_id: &str,
) -> Result<TokenMarketData, sqlx::Error> {
    db_access
        .get_token_marketdata(contract_address, chain_id, token_id)
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

#[allow(clippy::too_many_arguments)]
pub async fn get_token_offers_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    token_id: &str,
    page: i64,
    items_per_page: i64,
) -> Result<(Vec<TokenOfferOneDataDB>, bool, i64), sqlx::Error> {
    db_access
        .get_token_offers_data(contract_address, chain_id, token_id, page, items_per_page)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_token_activity_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    token_id: &str,
    page: i64,
    items_per_page: i64,
    direction: &str,
    types: &Option<Vec<TokenEventType>>,
) -> Result<(Vec<TokenActivityData>, bool, i64), sqlx::Error> {
    db_access
        .get_token_activity_data(
            contract_address,
            chain_id,
            token_id,
            page,
            items_per_page,
            direction,
            types,
        )
        .await
}

pub async fn flush_all_data_query<D: DatabaseAccess + Sync>(
    db_access: &D,
) -> Result<u64, sqlx::Error> {
    db_access.flush_all_data().await
}

pub async fn refresh_token_metadata<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
    token_id: &str,
) -> Result<(), sqlx::Error> {
    db_access
        .refresh_token_metadata(contract_address, chain_id, token_id)
        .await
}

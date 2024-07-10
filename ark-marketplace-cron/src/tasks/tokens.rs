use crate::models::token::TokenData;
use async_std::stream::StreamExt;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::json;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::HashSet;
use tracing::info;

const CHAIN_ID: &str = "0x534e5f4d41494e";
const ITEMS_PER_PAGE: i64 = 50;

#[derive(sqlx::FromRow)]
struct Offer {
    offer_amount: Option<f64>,
    order_hash: Option<String>,
    start_date: Option<i64>,
    end_date: Option<i64>,
    currency_address: Option<String>,
}

async fn clear_collection_cache(
    mut con: MultiplexedConnection,
    contract_address: &str,
) -> redis::RedisResult<()> {
    // Create a pattern for matching keys
    let pattern = format!("*{}_*", contract_address);

    // Collect keys matching the pattern
    let mut cmd = redis::cmd("SCAN");
    cmd.cursor_arg(0);
    cmd.arg("MATCH").arg(pattern);
    let mut keys: Vec<String> = vec![];
    {
        let mut iter = cmd.iter_async::<_>(&mut con).await?;
        while let Some(key) = iter.next().await {
            keys.push(key);
        }
    }

    // Delete keys and log the results
    if !keys.is_empty() {
        con.del(keys.clone()).await?;
    }

    Ok(())
}

pub async fn update_listed_tokens(pool: &PgPool, con: MultiplexedConnection) {
    let select_collections_query = r#"
        SELECT DISTINCT contract_address
        FROM token
        WHERE NOW() > to_timestamp(listing_end_date)
          AND listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#;

    let collections: Vec<String> = match sqlx::query(select_collections_query).fetch_all(pool).await
    {
        Ok(rows) => rows.iter().map(|row| row.get::<String, _>(0)).collect(),
        Err(e) => {
            tracing::error!("Failed to select collections: {}", e);
            return;
        }
    };

    let collections_clone = collections.clone();
    // loop through collections and clear cache
    for collection in &collections_clone {
        match clear_collection_cache(con.clone(), collection).await {
            Ok(_) => info!("Cache cleared for collection: {}", collection),
            Err(e) => tracing::error!("Failed to clear cache for collection {}: {}", collection, e),
        }
    }

    let clean_dates_query = r#"
        UPDATE token
        SET listing_start_date = NULL,
            listing_end_date = NULL,
            listing_timestamp = NULL,
            listing_start_amount = NULL,
            listing_end_amount = NULL,
            listing_currency_address = NULL,
            listing_currency_chain_id = NULL,
            listing_type = NULL,
            listing_orderhash = NULL
        WHERE NOW() > to_timestamp(listing_end_date)
          AND listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#;

    match sqlx::query(clean_dates_query).execute(pool).await {
        Ok(_) => info!("Cleanup of listing dates successful."),
        Err(e) => tracing::error!("Failed to clean up listing dates: {}", e),
    }

    // cache collections
    for collection in &collections_clone {
        match cache_collection_page(pool, &mut con.clone(), collection).await {
            Ok(_) => info!("Cache updated for collection: {}", collection),
            Err(e) => tracing::error!(
                "Failed to update cache for collection {}: {}",
                collection,
                e
            ),
        }
    }
}

pub async fn update_top_bid_tokens(pool: &PgPool, con: MultiplexedConnection) {
    let select_expired_offers_query = r#"
        SELECT DISTINCT contract_address, token_id
        FROM token_offer
        WHERE NOW() > to_timestamp(end_date);
    "#;

    let expired_offers: Vec<(String, String)> = match sqlx::query_as(select_expired_offers_query)
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to select expired offers: {}", e);
            return;
        }
    };

    // Delete these offers from the `token_offer` table
    for (contract_address, token_id) in &expired_offers {
        let delete_expired_offers_query = r#"
            DELETE FROM token_offer
            WHERE contract_address = $1
              AND token_id = $2
              AND NOW() > to_timestamp(end_date);
        "#;

        match sqlx::query(delete_expired_offers_query)
            .bind(contract_address)
            .bind(token_id)
            .execute(pool)
            .await
        {
            Ok(_) => info!("Deleted expired offers for token: {}", token_id),
            Err(e) => tracing::error!(
                "Failed to delete expired offers for token {}: {}",
                token_id,
                e
            ),
        }
    }

    // For each token whose offer has been deleted, check if there are valid offers left
    for (contract_address, token_id) in &expired_offers {
        let select_valid_offers_query = r#"
            SELECT
                CAST(hex_to_decimal(offer_amount) AS FLOAT8) as offer_amount,
                order_hash,
                start_date,
                end_date,
                currency_address,
                broker_id
            FROM token_offer
            WHERE contract_address = $1
              AND token_id = $2
              AND NOW() <= to_timestamp(end_date)
              AND STATUS = 'PLACED'
            ORDER BY hex_to_decimal(offer_amount) DESC
            LIMIT 1;
        "#;

        let valid_offer: Result<Offer, _> = sqlx::query_as(select_valid_offers_query)
            .bind(contract_address)
            .bind(token_id)
            .fetch_one(pool)
            .await;

        // Update `top_bid` fields based on whether a valid offer exists
        let update_top_bid_query = match valid_offer {
            Ok(offer) => format!(
                r#"
                UPDATE token
                SET top_bid_amount = '{}',
                    top_bid_order_hash = '{}',
                    top_bid_start_date = '{}',
                    top_bid_end_date = '{}',
                    top_bid_currency_address = '{}',
                    has_bid = true
                WHERE contract_address = '{}'
                  AND token_id = '{}';
                "#,
                offer.offer_amount.unwrap_or(0.0),
                offer.order_hash.unwrap_or_default(),
                offer.start_date.unwrap_or(0),
                offer.end_date.unwrap_or(0),
                offer.currency_address.unwrap_or_default(),
                contract_address,
                token_id
            ),
            _ => format!(
                r#"
                UPDATE token
                SET top_bid_amount = NULL,
                    top_bid_order_hash = NULL,
                    top_bid_start_date = NULL,
                    top_bid_end_date = NULL,
                    top_bid_currency_address = NULL,
                    top_bid_broker_id = NULL,
                    has_bid = false
                WHERE contract_address = '{}'
                  AND token_id = '{}';
            "#,
                contract_address, token_id
            ),
        };

        match sqlx::query(&update_top_bid_query).execute(pool).await {
            Ok(_) => info!(
                "Update of top_bid fields successful for token: {}",
                token_id
            ),
            Err(e) => tracing::error!(
                "Failed to update top_bid fields for token {}: {}",
                token_id,
                e
            ),
        }
    }

    // Clear the cache for each collection
    let collections: HashSet<String> = expired_offers
        .into_iter()
        .map(|(contract_address, _)| contract_address)
        .collect();
    for contract_address in &collections {
        match clear_collection_cache(con.clone(), contract_address).await {
            Ok(_) => info!("Cache cleared for collection: {}", contract_address),
            Err(e) => tracing::error!(
                "Failed to clear cache for collection {}: {}",
                contract_address,
                e
            ),
        }
    }

    // Rebuild the cache for each collection
    for contract_address in &collections {
        match cache_collection_page(pool, &mut con.clone(), contract_address).await {
            Ok(_) => info!("Cache updated for collection: {}", contract_address),
            Err(e) => tracing::error!(
                "Failed to update cache for collection {}: {}",
                contract_address,
                e
            ),
        }
    }
}

pub async fn cache_collection_pages(
    pool: &PgPool,
    con: MultiplexedConnection,
) -> redis::RedisResult<()> {
    let collections_to_cache = vec![
        "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af",
        "0x076503062d78f4481be03c9145022d6a4a71ec0719aa07756f79a2384dc7ef16",
        "0x0169e971d146ccf8f5e88f2b12e2e6099663fb56e42573479f2aee93309982f8",
    ];

    for contract_address in collections_to_cache {
        match cache_collection_page(pool, &mut con.clone(), contract_address).await {
            Ok(_) => info!("Successfully cached collection page"),
            Err(e) => tracing::error!("Failed to cache collection page: {}", e),
        }
    }

    Ok(())
}

async fn cache_collection_page(
    pool: &PgPool,
    con: &mut MultiplexedConnection,
    contract_address: &str,
) -> redis::RedisResult<()> {
    let total_token_count = sqlx::query!(
        "
            SELECT COUNT(*)
            FROM token
            WHERE token.contract_address = $1
              AND token.chain_id = $2
            ",
        contract_address,
        CHAIN_ID
    )
    .fetch_one(pool)
    .await;

    let token_count = match total_token_count {
        Ok(total_token_count) => total_token_count.count.unwrap_or(0),
        Err(e) => {
            tracing::error!("Failed to fetch token count: {}", e);
            0
        }
    };

    let total_pages = (token_count + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;

    for page in 1..=5 {
        let has_next_page = page < total_pages;

        let tokens_data: Vec<TokenData> = sqlx::query_as!(
            TokenData,
            "
               SELECT
                  token.contract_address as contract,
                  token.token_id,
                  hex_to_decimal(token.last_price) as last_price,
                  CAST(0 as INTEGER) as floor_difference,
                  token.listing_timestamp as listed_at,
                  hex_to_decimal(token.listing_start_amount) as price,
                  token.metadata as metadata
               FROM token
               WHERE token.contract_address = $3
                 AND token.chain_id = $4
               ORDER BY
                   token.listing_start_amount ASC NULLS LAST,
                   CAST(token.token_id AS NUMERIC)
           LIMIT $1 OFFSET $2",
            ITEMS_PER_PAGE,
            (page - 1) * ITEMS_PER_PAGE,
            contract_address,
            CHAIN_ID,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_else(|err| {
            tracing::error!("Error fetching data: {}", err);
            Vec::new()
        });
        let json_data = json!((tokens_data, has_next_page, token_count));
        let key = format!("all_tokens_{}_page_{}", contract_address, page);
        // Store the JSON data in Redis
        match con
            .set_ex::<_, _, ()>(&key, json_data.to_string(), 60)
            .await
        {
            Ok(_) => info!("Successfully set key"),
            Err(e) => tracing::error!("Failed to set key: {}", e),
        }
    }

    Ok(())
}

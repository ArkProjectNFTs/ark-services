use crate::models::token::TokenData;
use async_std::stream::StreamExt;
use chrono::Utc;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::types::BigDecimal;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::HashSet;
use tracing::info;

const CHAIN_ID: &str = "0x534e5f4d41494e";
const ITEMS_PER_PAGE: i64 = 50;
const REDIS_CACHE_TTL_SECONDS: u64 = 60;
const MAX_PAGES_TO_CACHE: i64 = 5;
const EXPIRED_OFFER_EVENT_TYPE: &str = "OfferExpired";
const EXPIRED_LISTING_EVENT_TYPE: &str = "ListingExpired";

#[derive(sqlx::FromRow)]
struct TokenOffer {
    broker_id: String,
    offer_amount: Option<f64>,
    order_hash: Option<String>,
    start_date: Option<i64>,
    end_date: Option<i64>,
    currency_address: Option<String>,
}

struct ExpiredOffer {
    contract_address: String,
    chain_id: String,
    token_id: String,
    order_hash: String,
    end_date: i64,
}

struct TokenEvent {
    token_event_id: String,
    token_id_hex: String,
    timestamp: i64,
}

async fn clear_collection_cache(
    con: &mut MultiplexedConnection,
    contract_address: &str,
) -> redis::RedisResult<()> {
    let cache_key_pattern = format!("*{}_*", contract_address);

    let mut cmd = redis::cmd("SCAN");
    cmd.cursor_arg(0);
    cmd.arg("MATCH").arg(cache_key_pattern);
    let mut keys: Vec<String> = vec![];
    {
        let mut iter = cmd.iter_async::<_>(con).await?;
        while let Some(key) = iter.next().await {
            keys.push(key);
        }
    }

    if !keys.is_empty() {
        // Explicitly specify the return type for del command
        let _: () = con.del(&keys).await?;
    }

    Ok(())
}

#[derive(FromRow)]
struct ExpiredListing {
    contract_address: String,
    chain_id: String,
    token_id: String,
    token_id_hex: String,
    listing_orderhash: Option<String>,
    listing_end_date: i64,
}

pub async fn update_listed_tokens(pool: &PgPool, con: &mut MultiplexedConnection) {
    let select_expired_listings_query = r#"
        SELECT DISTINCT contract_address, chain_id, token_id, token_id_hex, listing_orderhash, listing_end_date
        FROM token
        WHERE NOW() - interval '2 minutes' > to_timestamp(listing_end_date)
          AND listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#;

    let expired_listings: Vec<ExpiredListing> =
        match sqlx::query_as::<_, ExpiredListing>(select_expired_listings_query)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::error!("Failed to select expired listings: {}", e);
                return;
            }
        };

    for expired_listing in &expired_listings {
        let now = Utc::now().timestamp();
        let combined = format!(
            "{:?}{}",
            expired_listing.listing_orderhash, expired_listing.listing_end_date
        );
        let mut hasher = Sha256::new();
        hasher.update(combined);
        let result = hasher.finalize();
        let token_event_id = format!("0x{:x}", result);

        let insert_expired_listing_event_query = r#"
        INSERT INTO token_event (token_event_id, contract_address, chain_id, token_id, token_id_hex, event_type, block_timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7);
    "#;

        match sqlx::query(insert_expired_listing_event_query)
            .bind(&token_event_id)
            .bind(expired_listing.contract_address.clone())
            .bind(expired_listing.chain_id.clone())
            .bind(expired_listing.token_id.clone())
            .bind(expired_listing.token_id_hex.clone())
            .bind(EXPIRED_LISTING_EVENT_TYPE)
            .bind(now)
            .execute(pool)
            .await
        {
            Ok(_) => info!(
                "Inserted expired listing event for token: {}",
                expired_listing.token_id
            ),
            Err(e) => tracing::error!(
                "Failed to insert expired listing event for token {}: {}",
                expired_listing.token_id,
                e
            ),
        }
    }

    let collections: Vec<String> = match sqlx::query(
        r#"
    SELECT DISTINCT contract_address
    FROM token
    WHERE NOW() - interval '2 minutes' > to_timestamp(listing_end_date)
      AND listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows.iter().map(|row| row.get::<String, _>(0)).collect(),
        Err(e) => {
            tracing::error!("Failed to select collections: {}", e);
            return;
        }
    };

    for collection in &collections {
        if let Err(e) = clear_collection_cache(con, collection).await {
            tracing::error!("Failed to clear cache for collection {}: {}", collection, e);
            continue;
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
            listing_broker_id = NULL,
            listing_type = NULL,
            listing_orderhash = NULL
        WHERE contract_address = $1
          AND NOW() - interval '2 minutes' > to_timestamp(listing_end_date)
          AND listing_start_date IS NOT NULL 
          AND listing_end_date IS NOT NULL;
    "#;

        if let Err(e) = sqlx::query(clean_dates_query)
            .bind(collection)
            .execute(pool)
            .await
        {
            tracing::error!("Failed to clean up listing dates: {}", e);
            continue;
        }

        match sqlx::query_scalar::<_, BigDecimal>(
            r#"
        SELECT MIN(hex_to_decimal(listing_start_amount)) AS min_price
        FROM token
        WHERE contract_address = $1
          AND listing_start_date IS NOT NULL
          AND listing_end_date IS NOT NULL
        GROUP BY contract_address
        "#,
        )
        .bind(collection)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(min_price)) => {
                if let Err(e) = sqlx::query(
                    r#"
                UPDATE contract
                SET floor_price = $2
                WHERE contract_address = $1;
                "#,
                )
                .bind(collection)
                .bind(min_price)
                .execute(pool)
                .await
                {
                    tracing::error!(
                        "Failed to update floor price for collection {}: {}",
                        collection,
                        e
                    );
                }
            }
            Ok(None) => {
                if let Err(e) = sqlx::query(
                    r#"
                UPDATE contract
                SET floor_price = NULL
                WHERE contract_address = $1;
                "#,
                )
                .bind(collection)
                .execute(pool)
                .await
                {
                    tracing::error!(
                        "Failed to set floor price to NULL for collection {}: {}",
                        collection,
                        e
                    );
                }
            }
            Err(e) => tracing::error!(
                "Failed to recalculate floor price for collection {}: {}",
                collection,
                e
            ),
        }

        if let Err(e) = cache_collection_page(pool, con, collection).await {
            tracing::error!(
                "Failed to update cache for collection {}: {}",
                collection,
                e
            );
        }
    }
}

pub async fn update_top_bid_tokens(db_pool: &PgPool, redis_conn: &mut MultiplexedConnection) {
    let expired_offers = match get_expired_offers(db_pool).await {
        Ok(offers) => offers,
        Err(e) => {
            tracing::error!("Failed to select expired offers: {}", e);
            return;
        }
    };

    for offer in &expired_offers {
        if let Err(e) = process_expired_offer(db_pool, offer).await {
            tracing::error!("Failed to process expired offer: {}", e);
            continue;
        }
    }

    if let Err(e) = update_collections_cache(db_pool, redis_conn, &expired_offers).await {
        tracing::error!("Failed to update collections cache: {}", e);
    }
}

async fn get_expired_offers(pool: &PgPool) -> Result<Vec<ExpiredOffer>, sqlx::Error> {
    sqlx::query_as!(
        ExpiredOffer,
        r#"
        SELECT DISTINCT 
            contract_address, 
            chain_id, 
            token_id, 
            order_hash, 
            end_date
        FROM token_offer
        WHERE NOW() - interval '2 minutes' > to_timestamp(end_date)
        "#
    )
    .fetch_all(pool)
    .await
}

async fn process_expired_offer(pool: &PgPool, offer: &ExpiredOffer) -> Result<(), sqlx::Error> {
    let token_event = create_token_event(pool, offer).await?;

    insert_expired_event(pool, offer, &token_event).await?;
    delete_expired_offer(pool, offer).await?;
    update_token_bids(pool, offer).await?;

    Ok(())
}

async fn create_token_event(
    pool: &PgPool,
    offer: &ExpiredOffer,
) -> Result<TokenEvent, sqlx::Error> {
    let token_id_hex = sqlx::query_scalar!(
        r#"
        SELECT token_id_hex
        FROM token
        WHERE contract_address = $1
          AND chain_id = $2
          AND token_id = $3
        "#,
        offer.contract_address,
        offer.chain_id,
        offer.token_id
    )
    .fetch_one(pool)
    .await?;

    let token_event_id = {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", offer.order_hash, offer.end_date));
        format!("0x{:x}", hasher.finalize())
    };

    Ok(TokenEvent {
        token_event_id,
        token_id_hex,
        timestamp: Utc::now().timestamp(),
    })
}

async fn update_token_with_bid(
    pool: &PgPool,
    offer: &ExpiredOffer,
    bid: TokenOffer,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE token
        SET top_bid_amount = $1,
            top_bid_order_hash = $2,
            top_bid_start_date = $3,
            top_bid_end_date = $4,
            top_bid_currency_address = $5,
            top_bid_broker_id = $6,
            has_bid = true
        WHERE contract_address = $7
          AND token_id = $8
        "#,
    )
    .bind(bid.offer_amount)
    .bind(bid.order_hash)
    .bind(bid.start_date)
    .bind(bid.end_date)
    .bind(bid.currency_address)
    .bind(bid.broker_id)
    .bind(&offer.contract_address)
    .bind(&offer.token_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn clear_token_bid(pool: &PgPool, offer: &ExpiredOffer) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE token
        SET top_bid_amount = NULL,
            top_bid_order_hash = NULL,
            top_bid_start_date = NULL,
            top_bid_end_date = NULL,
            top_bid_currency_address = NULL,
            top_bid_broker_id = NULL,
            has_bid = false
        WHERE contract_address = $1
          AND token_id = $2
        "#,
    )
    .bind(&offer.contract_address)
    .bind(&offer.token_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_expired_event(
    pool: &PgPool,
    offer: &ExpiredOffer,
    event: &TokenEvent,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO token_event (
            token_event_id, contract_address, chain_id, token_id, 
            token_id_hex, event_type, block_timestamp
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(&event.token_event_id)
    .bind(&offer.contract_address)
    .bind(&offer.chain_id)
    .bind(&offer.token_id)
    .bind(&event.token_id_hex)
    .bind(EXPIRED_OFFER_EVENT_TYPE)
    .bind(event.timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

async fn delete_expired_offer(pool: &PgPool, offer: &ExpiredOffer) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM token_offer
        WHERE contract_address = $1
          AND chain_id = $2
          AND token_id = $3
          AND NOW() > to_timestamp(end_date)
        "#,
    )
    .bind(&offer.contract_address)
    .bind(&offer.chain_id)
    .bind(&offer.token_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn update_token_bids(pool: &PgPool, offer: &ExpiredOffer) -> Result<(), sqlx::Error> {
    let active_offer = sqlx::query_as::<_, TokenOffer>(
        r#"
        SELECT *
        FROM token_offer
        WHERE contract_address = $1
          AND token_id = $2
          AND chain_id = $3
          AND NOW() <= to_timestamp(end_date)
          AND STATUS = 'PLACED'
        ORDER BY hex_to_decimal(offer_amount) DESC
        LIMIT 1
        "#,
    )
    .bind(&offer.contract_address)
    .bind(&offer.token_id)
    .bind(&offer.chain_id)
    .fetch_optional(pool)
    .await?;

    match active_offer {
        Some(bid) => update_token_with_bid(pool, offer, bid).await?,
        None => clear_token_bid(pool, offer).await?,
    }

    Ok(())
}

async fn update_collections_cache(
    pool: &PgPool,
    redis_conn: &mut MultiplexedConnection,
    expired_offers: &[ExpiredOffer],
) -> Result<(), sqlx::Error> {
    let collections: HashSet<String> = expired_offers
        .iter()
        .map(|offer| offer.contract_address.clone())
        .collect();

    for contract_address in &collections {
        if let Err(e) = clear_collection_cache(redis_conn, contract_address).await {
            tracing::error!(
                "Failed to clear cache for collection {}: {}",
                contract_address,
                e
            );
            continue;
        }

        if let Err(e) = cache_collection_page(pool, redis_conn, contract_address).await {
            tracing::error!(
                "Failed to update cache for collection {}: {}",
                contract_address,
                e
            );
        }
    }

    Ok(())
}

pub async fn cache_collection_pages(
    db_pool: &PgPool,
    redis_conn: &mut MultiplexedConnection,
) -> redis::RedisResult<()> {
    let collections_to_cache = vec![
        "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af",
        "0x076503062d78f4481be03c9145022d6a4a71ec0719aa07756f79a2384dc7ef16",
        "0x0169e971d146ccf8f5e88f2b12e2e6099663fb56e42573479f2aee93309982f8",
    ];

    for contract_address in collections_to_cache {
        if let Err(e) = cache_collection_page(db_pool, redis_conn, contract_address).await {
            tracing::error!("Failed to cache collection page: {}", e);
        }
    }

    Ok(())
}

fn calculate_total_pages(total_items: i64, items_per_page: i64) -> i64 {
    (total_items + items_per_page - 1) / items_per_page
}

async fn cache_collection_page(
    db_pool: &PgPool,
    redis_conn: &mut MultiplexedConnection,
    contract_address: &str,
) -> redis::RedisResult<()> {
    let token_count_query = sqlx::query!(
        "
            SELECT COUNT(*)
            FROM token
            WHERE token.contract_address = $1
              AND token.chain_id = $2
            ",
        contract_address,
        CHAIN_ID
    )
    .fetch_one(db_pool)
    .await;

    let token_count = match token_count_query {
        Ok(total_token_count) => total_token_count.count.unwrap_or(0),
        Err(e) => {
            tracing::error!("Failed to fetch token count: {}", e);
            0
        }
    };

    let total_pages = calculate_total_pages(token_count, ITEMS_PER_PAGE);

    for page in 1..=MAX_PAGES_TO_CACHE {
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
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|err| {
            tracing::error!("Error fetching data: {}", err);
            Vec::new()
        });
        let json_data = json!((tokens_data, has_next_page, token_count));
        let cache_key = format!("all_tokens_{}_page_{}", contract_address, page);
        // Store the JSON data in Redis
        match redis_conn
            .set_ex::<_, _, ()>(&cache_key, json_data.to_string(), REDIS_CACHE_TTL_SECONDS)
            .await
        {
            Ok(_) => info!("Successfully set key"),
            Err(e) => tracing::error!("Failed to set key: {}", e),
        }
    }

    Ok(())
}

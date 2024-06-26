use crate::models::token::TokenData;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::json;
use sqlx::PgPool;
use tracing::info;

const CHAIN_ID: &str = "0x534e5f4d41494e";
const ITEMS_PER_PAGE: i64 = 50;

pub async fn update_listed_tokens(pool: &PgPool) {
    let update_is_listed_query = r#"
        UPDATE token
        SET is_listed = CASE
            WHEN NOW() BETWEEN to_timestamp(listing_start_date) AND to_timestamp(listing_end_date) THEN true
            ELSE false
        END
        WHERE listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#;

    match sqlx::query(update_is_listed_query).execute(pool).await {
        Ok(_) => info!("Update of is_listed field successful."),
        Err(e) => tracing::error!("Failed to update is_listed field: {}", e),
    }

    let clean_dates_query = r#"
        UPDATE token
        SET listing_start_date = NULL,
            listing_end_date = NULL
        WHERE (NOW() > to_timestamp(listing_end_date) OR NOW() < to_timestamp(listing_start_date))
          AND listing_start_date IS NOT NULL AND listing_end_date IS NOT NULL;
    "#;

    match sqlx::query(clean_dates_query).execute(pool).await {
        Ok(_) => info!("Cleanup of listing dates successful."),
        Err(e) => tracing::error!("Failed to clean up listing dates: {}", e),
    }
}

pub async fn update_top_bid_tokens(pool: &PgPool) {
    let update_top_bid_query = r#"
            UPDATE token
            SET top_bid_amount = (
                SELECT MAX(offer_amount)
                FROM token_offer
                WHERE
                    token_offer.contract_address = token.contract_address
                    AND token_offer.chain_id = token.chain_id
                    AND token_offer.token_id = token.token_id
                  AND EXTRACT(EPOCH FROM NOW()) BETWEEN start_date AND end_date
            ),
            top_bid_order_hash = (
                SELECT order_hash
                FROM token_offer
                WHERE
                    token_offer.contract_address = token.contract_address
                    AND token_offer.chain_id = token.chain_id
                    AND token_offer.token_id = token.token_id
                    AND offer_amount = (
                        SELECT MAX(offer_amount)
                        FROM token_offer
                        WHERE
                            token_offer.contract_address = token.contract_address
                            AND token_offer.chain_id = token.chain_id
                            AND token_offer.token_id = token.token_id
                            AND EXTRACT(EPOCH FROM NOW()) BETWEEN start_date AND end_date
                    )
                ORDER BY offer_timestamp ASC
                LIMIT 1
            );
        "#;

    match sqlx::query(update_top_bid_query).execute(pool).await {
        Ok(_) => info!("Update of top_bid field successful."),
        Err(e) => tracing::error!("Failed to update top_bid field: {}", e),
    }
}

pub async fn cache_collection_pages(
    pool: &PgPool,
    mut con: MultiplexedConnection,
) -> redis::RedisResult<()> {
    let collections_to_cache = vec![
        "0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af",
        "0x076503062d78f4481be03c9145022d6a4a71ec0719aa07756f79a2384dc7ef16",
        "0x0169e971d146ccf8f5e88f2b12e2e6099663fb56e42573479f2aee93309982f8",
    ];

    for contract_address in collections_to_cache {
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
                continue;
            }
        };

        let total_pages = (token_count + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;

        for page in 1..=5 {
            let has_next_page = page < total_pages;

            // all_tokens_{collection_id}_page_{page}
            let tokens_data: Vec<TokenData> = sqlx::query_as!(
                TokenData,
                "
                   SELECT
                       token.contract_address as contract,
                       token.token_id,
                       hex_to_decimal(token.last_price) as last_price,
                       CAST(0 as INTEGER) as floor_difference,
                       token.listing_timestamp as listed_at,
                       token.current_owner as owner,
                       token.block_timestamp as minted_at,
                       token.updated_timestamp as updated_at,
                       hex_to_decimal(token.listing_start_amount) as price,
                       token.metadata as metadata
                   FROM token
                   WHERE token.contract_address = $3
                     AND token.chain_id = $4
                   ORDER BY
                       CASE WHEN token.is_listed = true THEN 1 ELSE 2 END,
                       token.listing_start_amount ASC,
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
            match con.set::<_, _, ()>(&key, json_data.to_string()).await {
                Ok(_) => info!("Successfully set key"),
                Err(e) => tracing::error!("Failed to set key: {}", e),
            }
        }
    }

    Ok(())
}

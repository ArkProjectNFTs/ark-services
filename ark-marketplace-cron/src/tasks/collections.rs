use chrono::{TimeZone, Timelike, Utc};
use sqlx::PgPool;
use tracing::info;

pub async fn update_top_bid_collections(pool: &PgPool) {
    let update_top_bid_query = r#"
        UPDATE contract
        SET top_bid = (
            SELECT MAX(hex_to_decimal(offer_amount))
            FROM token_offer
            WHERE
                token_offer.contract_address = contract.contract_address
                AND token_offer.chain_id = contract.chain_id
                AND EXTRACT(EPOCH FROM NOW()) BETWEEN start_date AND end_date
        ),
        top_bid_order_hash = (
            SELECT order_hash
            FROM token_offer
            WHERE
                token_offer.contract_address = contract.contract_address
                AND token_offer.chain_id = contract.chain_id
                AND hex_to_decimal(offer_amount) = (
                    SELECT MAX(hex_to_decimal(offer_amount))
                    FROM token_offer
                    WHERE
                        token_offer.contract_address = contract.contract_address
                        AND token_offer.chain_id = contract.chain_id
                        AND EXTRACT(EPOCH FROM NOW()) BETWEEN start_date AND end_date
                )
            ORDER BY offer_timestamp ASC
            LIMIT 1
        );
    "#;

    match sqlx::query(update_top_bid_query).execute(pool).await {
        Ok(_) => {
            info!("Update of top_bid and top_bid_order_hash fields successful for collection.")
        }
        Err(e) => tracing::error!(
            "Failed to update top_bid and top_bid_order_hash fields for collection: {}",
            e
        ),
    }
}

pub async fn update_collections_market_data(pool: &PgPool) {
    let update_top_bid_query = r#"
        UPDATE contract
            SET
                token_count = (
                    SELECT COUNT(*)
                    FROM token
                    WHERE
                        token.contract_address = contract.contract_address
                        AND token.chain_id = contract.chain_id
                ),
                token_listed_count = (
                    SELECT COUNT(*)
                    FROM token
                    WHERE
                        token.contract_address = contract.contract_address
                        AND token.chain_id = contract.chain_id
                        AND token.listing_start_amount is not null
                ),
                listed_percentage = (
                    SELECT COUNT(*)
                        FROM token
                        WHERE token.contract_address = contract.contract_address
                        AND token.chain_id = contract.chain_id
                        AND token.listing_start_amount is not null
                    ) * 100 / NULLIF(
                        (
                            SELECT COUNT(*)
                            FROM token
                            WHERE token.contract_address = contract.contract_address
                            AND token.chain_id = contract.chain_id
                        ), 0
                ),
                volume_7d_eth = (
                    SELECT sum(hex_to_decimal(amount))
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sale'
                     AND token_event.block_timestamp >= (EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')::BIGINT)
                ),
                sales_7d = (
                    SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sale'
                     AND token_event.block_timestamp >= (EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')::BIGINT)
                ),
                owner_count = (
                    SELECT COUNT(*)
                     FROM (
                         SELECT current_owner
                         FROM token
                         WHERE contract_address = contract.contract_address
                           AND chain_id = contract.chain_id
                         GROUP BY current_owner
                     ) as owners
                ),
                total_volume = (
                    SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sale'
                ),
                total_sales = (
                    SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sale'
                ),
                marketcap = (
                    SELECT (MIN(hex_to_decimal(listing_start_amount)) * COUNT(*))
                     FROM token
                    WHERE
                        token.contract_address = contract.contract_address
                        AND token.chain_id = contract.chain_id
                ),
               floor_7d_percentage = (
                   COALESCE(
                        (
                            SELECT
                                (contract.floor_price - fc.floor) / NULLIF(fc.floor, 0) * 100
                            FROM
                                floor_collection fc
                            WHERE
                                fc.contract_address = contract.contract_address
                                AND fc.chain_id = contract.chain_id
                                AND to_timestamp(fc.timestamp) >= (CURRENT_DATE - INTERVAL '7 days')
                            ORDER BY
                                fc.timestamp ASC
                            LIMIT 1
                        ),
                        0
                    )
               )
    "#;

    match sqlx::query(update_top_bid_query).execute(pool).await {
        Ok(_) => info!("Update update_collections_market_data"),
        Err(e) => tracing::error!(
            "Failed to update update_collections_market_data field for collection: {}",
            e
        ),
    }
}

pub async fn insert_floor_price(pool: &PgPool) {
    let now = chrono::Utc::now();
    let current_hour = now.time().hour();
    let current_hour_start_naive = now
        .date_naive()
        .and_hms_opt(current_hour, 0, 0)
        .expect("Invalid time");
    let current_hour_start = Utc.from_utc_datetime(&current_hour_start_naive);
    let current_timestamp = current_hour_start.timestamp();

    let action_query = r#"
        WITH temp_min_price AS (
            SELECT
                contract_address,
                chain_id,
                DATE_TRUNC('hour', to_timestamp(listing_timestamp)) AS hour,
                MIN(hex_to_decimal(listing_start_amount)) AS min_price
            FROM
                token
            WHERE
                to_timestamp(listing_timestamp) <= to_timestamp($1) AND
                to_timestamp(listing_timestamp) > to_timestamp($1) - INTERVAL '24 hour'
            GROUP BY
                contract_address, chain_id, DATE_TRUNC('hour', to_timestamp(listing_timestamp))
        )
        INSERT INTO floor_collection (contract_address, chain_id, timestamp, floor)
        SELECT
            fc.contract_address,
            fc.chain_id,
            $1::bigint,
            COALESCE(
                (SELECT min_price FROM temp_min_price t WHERE t.contract_address = fc.contract_address AND t.chain_id = fc.chain_id ORDER BY t.hour DESC LIMIT 1),
                '0'
            )
        FROM (SELECT DISTINCT contract_address, chain_id FROM token) AS fc
        ON CONFLICT (contract_address, chain_id, timestamp)
        DO UPDATE SET floor = EXCLUDED.floor
    "#;

    match sqlx::query(action_query)
        .bind(current_timestamp as i64)
        .execute(pool)
        .await
    {
        Ok(_) => info!("Successfully inserted floor price for all collections."),
        Err(e) => tracing::error!("Failed to insert floor price for all collections: {}", e),
    }
}

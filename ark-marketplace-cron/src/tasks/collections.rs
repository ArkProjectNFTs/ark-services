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
                     SELECT
                        COALESCE(
                            ROUND(
                                SUM(
                                    hex_to_decimal(amount) / POWER(10::NUMERIC, COALESCE(cm.decimals, 18)::NUMERIC)
                                )::NUMERIC,
                                2
                            ),
                            0
                        )
                     FROM token_event
                     LEFT JOIN currency_mapping cm
                        ON token_event.contract_address = cm.currency_address
                        AND token_event.chain_id = cm.chain_id
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Executed'
                     AND token_event.block_timestamp >= (EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')::BIGINT)
                ),
                sales_7d = (
                    SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Executed'
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
                    SELECT
                        COALESCE(
                            ROUND(
                                SUM(
                                    hex_to_decimal(amount) / POWER(10::NUMERIC, COALESCE(cm.decimals, 18)::NUMERIC)
                                )::NUMERIC,
                                2
                            ),
                            0
                        )
                     FROM token_event
                     LEFT JOIN currency_mapping cm
                        ON token_event.contract_address = cm.currency_address
                        AND token_event.chain_id = cm.chain_id
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Executed'
                ),
                total_sales = (
                    SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Executed'
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
        WHERE market_data_enabled = true
    "#;

    match sqlx::query(update_top_bid_query).execute(pool).await {
        Ok(_) => info!("Update update_collections_market_data"),
        Err(e) => tracing::error!(
            "Failed to update update_collections_market_data field for collection: {}",
            e
        ),
    }
}

pub async fn update_contract_marketdata(pool: &PgPool) {
    let update_old_timestamps_query = r#"
        UPDATE contract
        SET calculate_marketdata_timestamp = NULL
        WHERE calculate_marketdata_timestamp < NOW() - INTERVAL '15 days';
    "#;

    match sqlx::query(update_old_timestamps_query).execute(pool).await {
        Ok(_) => {
            info!("Successfully updated calculate_marketdata_timestamp to NULL for old contracts.")
        }
        Err(e) => tracing::error!(
            "Failed to update calculate_marketdata_timestamp for contracts: {}",
            e
        ),
    }

    let time_ranges = ["10m", "1h", "6h", "1d", "7d", "30d"];
    for &time_range in &time_ranges {
        let query = format!(
            r#"
            INSERT INTO contract_marketdata (
                contract_address,
                chain_id,
                floor_percentage,
                volume,
                number_of_sales,
                timerange
            )
            SELECT
                contract.contract_address,
                contract.chain_id,
                COALESCE(
                    (
                        SELECT
                            (contract.floor_price - fc.floor) / NULLIF(fc.floor, 0) * 100
                        FROM
                            floor_collection fc
                        WHERE
                            fc.contract_address = contract.contract_address
                            AND fc.chain_id = contract.chain_id
                            AND to_timestamp(fc.timestamp) >= (CURRENT_DATE - INTERVAL '{}')
                        ORDER BY
                            fc.timestamp ASC
                        LIMIT 1
                    ),
                    0
                ) AS floor_percentage,
                COALESCE(
                    (
                        SELECT
                            SUM(CAST(amount AS BIGINT))
                        FROM
                            token_event
                        WHERE
                            token_event.contract_address = contract.contract_address
                            AND token_event.chain_id = contract.chain_id
                            AND token_event.event_type = 'Executed'
                            AND to_timestamp(token_event.block_timestamp) >= (CURRENT_DATE - INTERVAL '{}')
                    ),
                    0
                ) AS volume,
                (
                    SELECT
                        COUNT(*)
                    FROM
                        token_event
                    WHERE
                        token_event.contract_address = contract.contract_address
                        AND token_event.chain_id = contract.chain_id
                        AND token_event.event_type = 'Executed'
                        AND to_timestamp(token_event.block_timestamp) >= (CURRENT_DATE - INTERVAL '{}')
                ) AS number_of_sales,
                '{}' AS timerange
            FROM
                contract
            WHERE contract.is_verified = true AND contract.market_data_enabled = true
            OR contract.calculate_marketdata_timestamp is not null
            ON CONFLICT (contract_address, chain_id, timerange)
            DO UPDATE SET
                floor_percentage = EXCLUDED.floor_percentage,
                volume = EXCLUDED.volume,
                number_of_sales = EXCLUDED.number_of_sales;
            "#,
            time_range, time_range, time_range, time_range
        );

        match sqlx::query(&query).execute(pool).await {
            Ok(_) => info!(
                "Successfully updated contract_marketdata for timerange: {}",
                time_range
            ),
            Err(e) => tracing::error!(
                "Failed to update contract_marketdata for timerange {}: {}",
                time_range,
                e
            ),
        }
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
        .bind(current_timestamp)
        .execute(pool)
        .await
    {
        Ok(_) => info!("Successfully inserted floor price for all collections."),
        Err(e) => tracing::error!("Failed to insert floor price for all collections: {}", e),
    }
}

pub async fn empty_floor_price(pool: &PgPool) {
    let action_query = r#"
        WITH empty_collections AS (
            SELECT
                contract_address,
                chain_id
            FROM
                token
            WHERE
                listing_start_amount IS NULL
                AND listing_end_amount IS NULL
            GROUP BY
                contract_address, chain_id
            HAVING COUNT(*) = (
                SELECT COUNT(*)
                FROM token t
                WHERE t.contract_address = token.contract_address
                  AND t.chain_id = token.chain_id
            )
        )
        UPDATE contract
        SET floor_price = NULL
        WHERE (contract_address, chain_id) IN (SELECT contract_address, chain_id FROM empty_collections)
          AND floor_price IS NOT NULL
    "#;

    match sqlx::query(action_query).execute(pool).await {
        Ok(_) => info!("Successfully updated floor price for empty collections."),
        Err(e) => tracing::error!("Failed to update floor price for empty collections: {}", e),
    }
}

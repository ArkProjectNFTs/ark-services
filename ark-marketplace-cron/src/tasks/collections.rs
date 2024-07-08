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

pub async fn update_collections_floor(pool: &PgPool) {
    let update_top_bid_query = r#"
        UPDATE contract
        SET floor_price = min_price
        FROM (
                 SELECT contract_address, chain_id, MIN(hex_to_decimal(listing_start_amount)) as min_price
                 FROM token
                 GROUP BY contract_address, chain_id
             ) AS token
        WHERE token.contract_address = contract.contract_address
          AND token.chain_id = contract.chain_id;
    "#;

    match sqlx::query(update_top_bid_query).execute(pool).await {
        Ok(_) => info!("Update floor_price"),
        Err(e) => tracing::error!("Failed to update floor_price field for collection: {}", e),
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
                );
    "#;

    match sqlx::query(update_top_bid_query).execute(pool).await {
        Ok(_) => info!("Update update_collections_market_data"),
        Err(e) => tracing::error!(
            "Failed to update update_collections_market_data field for collection: {}",
            e
        ),
    }
}

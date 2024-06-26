use sqlx::PgPool;
use tracing::info;

pub async fn update_top_bid_collections(pool: &PgPool) {
    let update_top_bid_query = r#"
        UPDATE contract
        SET top_bid = (
            SELECT MAX(offer_amount)
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
                AND offer_amount = (
                    SELECT MAX(offer_amount)
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
        SET floor_price = (
            SELECT MIN(listing_start_amount)
            FROM token
            WHERE
                token.contract_address = contract.contract_address
                AND token.chain_id = contract.chain_id
        );
    "#;

    match sqlx::query(update_top_bid_query).execute(pool).await {
        Ok(_) => info!("Update floor_price"),
        Err(e) => tracing::error!("Failed to update floor_price field for collection: {}", e),
    }
}

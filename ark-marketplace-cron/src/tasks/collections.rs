use crate::models::token::TokenData;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::json;
use sqlx::PgPool;
use tracing::info;

const CHAIN_ID: &str = "0x534e5f4d41494e";
const ITEMS_PER_PAGE: i64 = 50;

pub async fn update_top_bid_tokens_for_collection(pool: &PgPool, contract_address: &str, chain_id: &str) {
    let update_top_bid_query = r#"
        UPDATE token
        SET
            top_bid = (
                SELECT MAX(bid_amount)
                FROM token_bid
                WHERE
                    token_bid.contract_address = token.contract_address
                    AND token_bid.chain_id = token.chain_id
                    AND token_bid.token_id = token.token_id
                    AND EXTRACT(EPOCH FROM NOW()) BETWEEN start_date AND end_date
            ),
            top_bid_order_hash = (
                SELECT order_hash
                FROM token_bid
                WHERE
                    token_bid.contract_address = token.contract_address
                    AND token_bid.chain_id = token.chain_id
                    AND token_bid.token_id = token.token_id
                    AND bid_amount = (
                        SELECT MAX(bid_amount)
                        FROM token_bid
                        WHERE
                            token_bid.contract_address = token.contract_address
                            AND token_bid.chain_id = token.chain_id
                            AND token_bid.token_id = token.token_id
                            AND EXTRACT(EPOCH FROM NOW()) BETWEEN start_date AND end_date
                    )
            )
        WHERE
            token.contract_address = $1
            AND token.chain_id = $2;
    "#;

    match sqlx::query(update_top_bid_query)
        .bind(contract_address)
        .bind(chain_id)
        .execute(pool)
        .await {
        Ok(_) => info!("Update of top_bid and top_bid_order_hash fields successful for collection."),
        Err(e) => tracing::error!("Failed to update top_bid and top_bid_order_hash fields for collection: {}", e),
    }
}

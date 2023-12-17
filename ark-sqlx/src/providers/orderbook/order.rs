use arkproject::diri::storage::types::{CancelledData, ExecutedData, FulfilledData, PlacedData};
use std::fmt;
use tracing::trace;

use crate::providers::{ProviderError, SqlxCtx};

#[derive(Debug, Copy, Clone)]
pub enum OrderStatus {
    Placed,
    Fulfilled,
    Cancelled,
    Executed,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderStatus::Placed => "PLACED",
                OrderStatus::Fulfilled => "FULFILLED",
                OrderStatus::Cancelled => "CANCELLED",
                OrderStatus::Executed => "EXECUTED",
            }
        )
    }
}

pub struct OrderProvider {}

impl OrderProvider {
    pub async fn update_order_status(
        client: &SqlxCtx,
        block_id: u64,
        block_timestamp: u64,
        order_hash: &str,
        status: OrderStatus,
    ) -> Result<(), ProviderError> {
        trace!("Updating order status {} {}", order_hash, status);

        let q = "INSERT INTO orderbook_order_status (block_id, block_timestamp, order_hash, status) VALUES ($1, $2, $3, $4) ON CONFLICT (order_hash) DO UPDATE SET block_id = $1, block_timestamp = $2, status = $4";

        let _r = sqlx::query(q)
            .bind(block_id as i64)
            .bind(block_timestamp as i64)
            .bind(order_hash.to_string())
            .bind(status.to_string())
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    pub async fn register_placed(
        client: &SqlxCtx,
        block_id: u64,
        block_timestamp: u64,
        data: &PlacedData,
    ) -> Result<(), ProviderError> {
        trace!("Registering placed order {:?}", data);

        let q = "INSERT INTO orderbook_order_placed (block_id, block_timestamp, order_hash, order_version, order_type, cancelled_order_hash, route, currency_address, currency_chain_id, salt, offerer, token_chain_id, token_address, token_id, quantity, start_amount, end_amount, start_date, end_date, broker_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20);";

        let _r = sqlx::query(q)
            .bind(block_id as i64)
            .bind(block_timestamp as i64)
            .bind(data.order_hash.clone())
            .bind(data.order_version.clone())
            .bind(data.order_type.clone())
            .bind(data.cancelled_order_hash.clone())
            .bind(data.route.clone())
            .bind(data.currency_address.clone())
            .bind(data.currency_chain_id.clone())
            .bind(data.salt.clone())
            .bind(data.offerer.clone())
            .bind(data.token_chain_id.clone())
            .bind(data.token_address.clone())
            .bind(data.token_id.clone())
            .bind(data.quantity.clone())
            .bind(data.start_amount.clone())
            .bind(data.end_amount.clone())
            .bind(data.start_date as i64)
            .bind(data.end_date as i64)
            .bind(data.broker_id.clone())
            .execute(&client.pool)
            .await?;

        Self::update_order_status(
            client,
            block_id,
            block_timestamp,
            &data.order_hash,
            OrderStatus::Placed,
        )
        .await?;

        Ok(())
    }

    pub async fn register_cancelled(
        client: &SqlxCtx,
        block_id: u64,
        block_timestamp: u64,
        data: &CancelledData,
    ) -> Result<(), ProviderError> {
        trace!("Registering cancelled order {:?}", data);

        let q = "INSERT INTO orderbook_order_cancelled (block_id, block_timestamp, order_hash, reason) VALUES ($1, $2, $3, $4);";

        let _r = sqlx::query(q)
            .bind(block_id as i64)
            .bind(block_timestamp as i64)
            .bind(data.order_hash.clone())
            .bind(data.reason.clone())
            .execute(&client.pool)
            .await?;

        Self::update_order_status(
            client,
            block_id,
            block_timestamp,
            &data.order_hash,
            OrderStatus::Cancelled,
        )
        .await?;

        Ok(())
    }

    pub async fn register_fulfilled(
        client: &SqlxCtx,
        block_id: u64,
        block_timestamp: u64,
        data: &FulfilledData,
    ) -> Result<(), ProviderError> {
        trace!("Registering fulfilled order {:?}", data);

        let q = "INSERT INTO orderbook_order_fulfilled (block_id, block_timestamp, order_hash, fulfiller, related_order_hash) VALUES ($1, $2, $3, $4, $5);";

        let _r = sqlx::query(q)
            .bind(block_id as i64)
            .bind(block_timestamp as i64)
            .bind(data.order_hash.clone())
            .bind(data.fulfiller.clone())
            .bind(data.related_order_hash.clone())
            .execute(&client.pool)
            .await?;

        Self::update_order_status(
            client,
            block_id,
            block_timestamp,
            &data.order_hash,
            OrderStatus::Fulfilled,
        )
        .await?;

        Ok(())
    }

    pub async fn register_executed(
        client: &SqlxCtx,
        block_id: u64,
        block_timestamp: u64,
        data: &ExecutedData,
    ) -> Result<(), ProviderError> {
        trace!("Registering executed order {:?}", data);

        let q = "INSERT INTO orderbook_order_executed (block_id, block_timestamp, order_hash) VALUES ($1, $2, $3);";

        let _r = sqlx::query(q)
            .bind(block_id as i64)
            .bind(block_timestamp as i64)
            .bind(data.order_hash.clone())
            .execute(&client.pool)
            .await?;

        Self::update_order_status(
            client,
            block_id,
            block_timestamp,
            &data.order_hash,
            OrderStatus::Executed,
        )
        .await?;

        Ok(())
    }
}

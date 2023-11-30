use arkproject::diri::storage::types::NewOrderData;
use tracing::trace;

use crate::providers::{ProviderError, SqlxCtx};

pub struct OrderProvider {}

impl OrderProvider {
    pub async fn add_new_order(
        client: &SqlxCtx,
        block_id: u64,
        data: &NewOrderData,
    ) -> Result<(), ProviderError> {
        trace!("Registering new order {:?}", data);

        let q = "INSERT INTO orderbook_order (order_hash, order_version, order_type, cancelled_order_hash, route, currency_address, currency_chain_id, salt, offerer, token_chain_id, token_address, token_id, quantity, start_amount, end_amount, start_date, end_date, broker_id, block_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19);";

        let _r = sqlx::query(q)
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
            .bind(block_id as i64)
            .execute(&client.pool)
            .await?;

        Ok(())
    }
}

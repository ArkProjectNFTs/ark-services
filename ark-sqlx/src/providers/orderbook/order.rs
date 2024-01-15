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

#[derive(Debug, Copy, Clone)]
pub enum EventType {
    Listing,
    Auction,
    Offer,
    CollectionOffer,
}

impl EventType {
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        match s {
            "Listing" => Ok(EventType::Listing),
            "Auction" => Ok(EventType::Auction),
            "Offer" => Ok(EventType::Offer),
            "CollectionOffer" => Ok(EventType::CollectionOffer),
            _ => Err("Unknown event type"),
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EventType::Listing => "Listing",
                EventType::Auction => "Auction",
                EventType::Offer => "Offer",
                EventType::CollectionOffer => "CollectionOffer",
            }
        )
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

struct EventHistoryData {
    token_id: String,
    token_address: String,
    event_type: EventType,
    order_status: OrderStatus,
    event_timestamp: i64,
    previous_owner: Option<String>,
    new_owner: Option<String>,
    amount: Option<String>,
    canceled_reason: Option<String>,
}

struct TokenData {
    token_id: String,
    token_address: String,
    order_type: EventType,
}

impl OrderProvider {

    pub async fn get_token_data_by_order_hash(
        client: &SqlxCtx,
        order_hash: &str,
    ) -> Result<TokenData, ProviderError> {
        let query = "
        SELECT token_id, token_address, order_type
        FROM orderbook_order_created
        WHERE order_hash = $1;
    ";

        let token_data = sqlx::query_as::<_, TokenData>(query)
            .bind(order_hash)
            .fetch_one(&client.pool)
            .await?;

        Ok(token_data)
    }

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

    async fn insert_event_history(
        client: &SqlxCtx,
        event_data: &EventHistoryData,
    ) -> Result<(), ProviderError> {
        trace!("Insert event history");

        let q = "
            INSERT INTO orderbook_token_history (token_id, token_address, event_type, event_timestamp, previous_owner, new_owner, amount, additional_info)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8);
        ";

        let _r = sqlx::query(q)
            .bind(&event_data.token_id)
            .bind(&event_data.token_address)
            .bind(event_data.event_type.to_string())
            .bind(event_data.event_timestamp)
            .bind(&event_data.previous_owner)
            .bind(&event_data.new_owner)
            .bind(&event_data.amount)
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

        let q = "INSERT INTO orderbook_order_created (block_id, block_timestamp, order_hash, order_version, order_type, cancelled_order_hash, route, currency_address, currency_chain_id, salt, offerer, token_chain_id, token_address, token_id, quantity, start_amount, end_amount, start_date, end_date, broker_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20);";

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

        // insert token only for the first listing
        let upsert_query = "
            INSERT INTO orderbook_token (token_chain_id, token_address, token_id, listed_timestamp, updated_timestamp, status, current_owner, current_amount, quantity, start_amount, end_amount, start_date, end_date, broker_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (token_id, token_address)
            DO NOTHING;
        ";

        sqlx::query(upsert_query)
            .bind(data.token_chain_id.clone())
            .bind(data.token_address.clone())
            .bind(data.token_id.clone())
            .bind(block_timestamp as i64)
            .bind(block_timestamp as i64)
                .bind(OrderStatus::Placed.to_string())
            .bind(data.offerer.clone())
            .bind(data.start_amount.clone())
            .execute(&client.pool)
            .await?;

        let event_type = EventType::from_str(&data.order_type).map_err(ProviderError::from)?;

        Self::insert_event_history(
            client,
            &EventHistoryData {
                token_id: data.token_id.clone().expect("Missing token id"),
                token_address: data.token_address.clone(),
                event_type,
                event_timestamp: block_timestamp as i64,
                order_status: OrderStatus::Placed,
                previous_owner: None,
                new_owner: Some(data.offerer.clone()),
                amount: Some(data.start_amount.clone()),
                canceled_reason: None,
            },
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

        let token_data = Self::get_token_data_by_order_hash(client, &data.order_hash).await?;
        Self::insert_event_history(
            client,
            &EventHistoryData {
                token_id: token_data.token_id,
                token_address: token_data.token_address,
                event_type: token_data.order_type.into(),
                event_timestamp: block_timestamp as i64,
                order_status: OrderStatus::Cancelled,
                canceled_reason: Some(data.reason.clone()),
                new_owner: None,
                amount: None,
                previous_owner: None,
            }
        ).await?;

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

        let token_data = Self::get_token_data_by_order_hash(client, &data.order_hash).await?;
        Self::insert_event_history(
            client,
            &EventHistoryData {
                token_id: token_data.token_id,
                token_address: token_data.token_address,
                event_type: token_data.order_type.into(),
                order_status: OrderStatus::Fulfilled,
                event_timestamp: block_timestamp as i64,
                canceled_reason: Some(data.reason.clone()),
                new_owner: None,
                amount: None,
                previous_owner: None,
            }
        ).await?;

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

        let token_data = Self::get_token_data_by_order_hash(client, &data.order_hash).await?;
        Self::insert_event_history(
            client,
            &EventHistoryData {
                token_id: token_data.token_id,
                token_address: token_data.token_address,
                event_type: token_data.order_type.into(),
                order_status: OrderStatus::Executed,
                event_timestamp: block_timestamp as i64,
                canceled_reason: Some(data.reason.clone()),
                new_owner: None,
                amount: None,
                previous_owner: None,
            }
        ).await?;

        Ok(())
    }
}

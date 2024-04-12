use arkproject::diri::storage::types::{
    CancelledData, ExecutedData, FulfilledData, PlacedData, RollbackStatusData,
};
use sqlx::Row;
use std::fmt;
use std::str::FromStr;
use tracing::{error, trace};

use crate::providers::{ProviderError, SqlxCtx};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum RollbackStatus {
    CancelledUser,
    CancelledByNewOrder,
    CancelledAssetFault,
    CancelledOwnership,
}

impl RollbackStatus {
    fn from_code(code: u32) -> Option<RollbackStatus> {
        match code {
            1 => Some(RollbackStatus::CancelledUser),
            2 => Some(RollbackStatus::CancelledByNewOrder),
            3 => Some(RollbackStatus::CancelledAssetFault),
            4 => Some(RollbackStatus::CancelledOwnership),
            _ => None,
        }
    }
}

impl fmt::Display for RollbackStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            RollbackStatus::CancelledUser => "CANCELLED_USER",
            RollbackStatus::CancelledByNewOrder => "CANCELLED_NEW_ORDER",
            RollbackStatus::CancelledAssetFault => "CANCELLED_ASSET_FAULT",
            RollbackStatus::CancelledOwnership => "CANCELLED_OWNERSHIP",
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OrderStatus {
    Placed,
    Fulfilled,
    Cancelled,
    Executed,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EventType {
    Listing,
    Auction,
    Offer,
    CollectionOffer,
}

impl From<String> for EventType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Listing" => EventType::Listing,
            "Auction" => EventType::Auction,
            "Offer" => EventType::Offer,
            "CollectionOffer" => EventType::CollectionOffer,
            _ => {
                error!("Unknown event type: {}", s);
                EventType::Listing
            }
        }
    }
}

impl FromStr for EventType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Listing" => Ok(EventType::Listing),
            "Auction" => Ok(EventType::Auction),
            "Offer" => Ok(EventType::Offer),
            "CollectionOffer" => Ok(EventType::CollectionOffer),
            _ => Err("Unknown event type"),
        }
    }
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Listing => "Listing",
            EventType::Auction => "Auction",
            EventType::Offer => "Offer",
            EventType::CollectionOffer => "CollectionOffer",
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
    end_amount: Option<String>,
    start_date: Option<i64>,
    end_date: Option<i64>,
}

struct OfferData {
    order_hash: String,
    token_id: String,
    token_address: String,
    timestamp: i64,
    maker: String,
    amount: String,
    quantity: String,
    currency_chain_id: String,
    currency_address: String,
    start_date: i64,
    end_date: i64,
    status: String,
}

pub struct OfferExecutedInfo {
    block_timestamp: u64,
    token_address: String,
    token_id: String,
    new_owner: String,
    price: String,
    order_hash: String,
    currency_chain_id: String,
    currency_address: String,
}

#[derive(Debug)]
pub struct TokenData {
    token_id: String,
    token_address: String,
    order_type: String,
    offerer: String,
    start_amount: String,
    order_hash: String,
    currency_chain_id: String,
    currency_address: String,
}

impl OrderProvider {
    pub async fn get_token_data_by_order_hash(
        client: &SqlxCtx,
        order_hash: &str,
    ) -> Result<Option<TokenData>, sqlx::Error> {
        let query = "
            SELECT token_id, token_address, order_type, offerer, start_amount, order_hash, currency_chain_id, currency_address
            FROM orderbook_order_created
            WHERE order_hash = $1;
        ";

        if let Some((
            token_id,
            token_address,
            order_type,
            offerer,
            start_amount,
            order_hash,
            currency_chain_id,
            currency_address,
        )) = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            ),
        >(query)
        .bind(order_hash)
        .fetch_optional(&client.pool)
        .await?
        {
            Ok(Some(TokenData {
                token_id,
                token_address,
                order_type,
                offerer,
                start_amount,
                order_hash,
                currency_chain_id,
                currency_address,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_current_owner(
        client: &SqlxCtx,
        token_address: &str,
        token_id: &str,
    ) -> Result<String, ProviderError> {
        let query = "
            SELECT current_owner
            FROM orderbook_token
            WHERE token_address = $1 AND token_id = $2;
        ";
        let result = sqlx::query(query)
            .bind(token_address)
            .bind(token_id)
            .fetch_one(&client.pool)
            .await?;

        let current_owner: String = result.get::<String, _>("current_owner");

        Ok(current_owner)
    }

    pub async fn update_token_status(
        client: &SqlxCtx,
        token_address: &str,
        token_id: &str,
        status: OrderStatus,
    ) -> Result<(), ProviderError> {
        let query = "
        UPDATE orderbook_token
        SET
            status = $3
        WHERE token_address = $1 AND token_id = $2;
        ";

        sqlx::query(query)
            .bind(token_address)
            .bind(token_id)
            .bind(status.to_string())
            .execute(&client.pool)
            .await?;

        // if status is fulfilled, then buy_in_progress should be set to true
        let buy_in_progress = status == OrderStatus::Fulfilled;
        let query = "
        UPDATE orderbook_token
        SET
            buy_in_progress = $3
        WHERE token_address = $1 AND token_id = $2;
        ";

        sqlx::query(query)
            .bind(token_address)
            .bind(token_id)
            .bind(buy_in_progress)
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    pub async fn update_offer_status(
        client: &SqlxCtx,
        order_hash: &str,
        status: OrderStatus,
    ) -> Result<(), ProviderError> {
        let query = "
            UPDATE orderbook_token_offers
            SET
                status = $2
            WHERE order_hash = $1;
        ";

        sqlx::query(query)
            .bind(order_hash)
            .bind(status.to_string())
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    pub async fn clear_token_data_if_listing(
        client: &SqlxCtx,
        token_address: &str,
        token_id: &str,
        order_type: &str,
    ) -> Result<(), ProviderError> {
        let event_type = EventType::from_str(order_type).map_err(ProviderError::from)?;
        if event_type == EventType::Listing {
            let query = "
            UPDATE orderbook_token
            SET
                start_amount = null,
                end_amount = null,
                start_date = null,
                end_date = null
            WHERE token_address = $1 AND token_id = $2;
            ";

            sqlx::query(query)
                .bind(token_address)
                .bind(token_id)
                .execute(&client.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn update_owner_price_on_offer_executed(
        client: &SqlxCtx,
        info: &OfferExecutedInfo,
    ) -> Result<(), ProviderError> {
        let query = "
            UPDATE orderbook_token
            SET
                current_owner = $3, updated_timestamp = $4,
                last_price = $5, order_hash = $6,
                currency_chain_id = $7, currency_address = $8,
                start_date = null, end_date = null,
                start_amount = null, end_amount = null
            WHERE token_address = $1 AND token_id = $2;
        ";

        sqlx::query(query)
            .bind(&info.token_address)
            .bind(&info.token_id)
            .bind(&info.new_owner)
            .bind(info.block_timestamp as i64)
            .bind(&info.price)
            .bind(&info.order_hash.to_string())
            .bind(&info.currency_chain_id)
            .bind(&info.currency_address)
            .execute(&client.pool)
            .await?;

        // to hide offers belonging to old owner
        sqlx::query("update orderbook_token_offers set start_date = null, end_date = null WHERE offer_maker = $1 AND token_address = $2 AND token_id = $3 and status != 'EXECUTED'")
            .bind(&info.new_owner)
            .bind(&info.token_address)
            .bind(&info.token_id)
            .execute(&client.pool)
            .await?;
        Ok(())
    }

    pub async fn update_token_on_listing_executed(
        client: &SqlxCtx,
        token_address: &str,
        token_id: &str,
        block_timestamp: i64,
        price: &str,
    ) -> Result<(), ProviderError> {
        let query = "
            UPDATE orderbook_token
            SET
                start_date = null, end_date = null,
                start_amount = null, end_amount = null,
                updated_timestamp = $3,
                last_price = $4
            WHERE token_address = $1 AND token_id = $2;
        ";

        sqlx::query(query)
            .bind(token_address)
            .bind(token_id)
            .bind(block_timestamp)
            .bind(price)
            .execute(&client.pool)
            .await?;

        Ok(())
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
            INSERT INTO orderbook_token_history (token_id, token_address, event_type, order_status, event_timestamp, previous_owner, new_owner, amount, canceled_reason, end_amount, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);
        ";

        let _r = sqlx::query(q)
            .bind(&event_data.token_id)
            .bind(&event_data.token_address)
            .bind(event_data.event_type.to_string())
            .bind(event_data.order_status.to_string())
            .bind(event_data.event_timestamp)
            .bind(&event_data.previous_owner.clone().unwrap_or_default())
            .bind(&event_data.new_owner.clone().unwrap_or_default())
            .bind(&event_data.amount.clone().unwrap_or_default())
            .bind(&event_data.canceled_reason.clone().unwrap_or_default())
            .bind(&event_data.end_amount.clone().unwrap_or_default())
            .bind(event_data.start_date.unwrap_or_default())
            .bind(event_data.end_date.unwrap_or_default())
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    async fn insert_offers(client: &SqlxCtx, offer_data: &OfferData) -> Result<(), ProviderError> {
        trace!("Insert token offers");
        let insert_query = "
            INSERT INTO orderbook_token_offers (token_id, token_address, offer_maker, offer_amount, offer_quantity, offer_timestamp, order_hash, currency_chain_id, currency_address, start_date, end_date, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);
        ";

        sqlx::query(insert_query)
            .bind(&offer_data.token_id)
            .bind(&offer_data.token_address)
            .bind(&offer_data.maker)
            .bind(&offer_data.amount)
            .bind(&offer_data.quantity)
            .bind(offer_data.timestamp)
            .bind(&offer_data.order_hash)
            .bind(&offer_data.currency_chain_id)
            .bind(&offer_data.currency_address)
            .bind(offer_data.start_date)
            .bind(offer_data.end_date)
            .bind(&offer_data.status)
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
            .bind(data.cancelled_order_hash.clone().unwrap_or_default())
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

        let event_type = EventType::from_str(&data.order_type).map_err(ProviderError::from)?;

        if event_type == EventType::Offer || event_type == EventType::CollectionOffer {
            // create token without listing information
            let upsert_query = "
                INSERT INTO orderbook_token (token_chain_id, token_address, token_id, listed_timestamp, updated_timestamp, order_hash)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (token_id, token_address)
                DO NOTHING;
            ";

            sqlx::query(upsert_query)
                .bind(data.token_chain_id.clone())
                .bind(data.token_address.clone())
                .bind(data.token_id.clone())
                .bind(block_timestamp as i64)
                .bind(block_timestamp as i64)
                .bind(data.order_hash.clone())
                .execute(&client.pool)
                .await?;

            Self::insert_offers(
                client,
                &OfferData {
                    token_id: data.token_id.clone().expect("Missing token id"),
                    token_address: data.token_address.clone(),
                    timestamp: block_timestamp as i64,
                    maker: data.offerer.clone(),
                    amount: data.start_amount.clone(),
                    quantity: data.quantity.clone(),
                    order_hash: data.order_hash.clone(),
                    start_date: data.start_date as i64,
                    end_date: data.end_date as i64,
                    currency_chain_id: data.currency_chain_id.clone(),
                    currency_address: data.currency_address.clone(),
                    status: OrderStatus::Placed.to_string(),
                },
            )
            .await?;
        } else {
            // create token with listing information
            let upsert_query = "
                INSERT INTO orderbook_token (token_chain_id, token_address, token_id, listed_timestamp, updated_timestamp, current_owner, quantity, start_amount, end_amount, start_date, end_date, broker_id, order_hash, currency_address, currency_chain_id, status)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT (token_id, token_address) DO UPDATE SET
                current_owner = EXCLUDED.current_owner,
                start_amount = EXCLUDED.start_amount,
                end_amount = EXCLUDED.end_amount,
                start_date = EXCLUDED.start_date,
                end_date = EXCLUDED.end_date,
                broker_id = EXCLUDED.broker_id,
                order_hash = EXCLUDED.order_hash,
                status = EXCLUDED.status,
                updated_timestamp = EXCLUDED.updated_timestamp;
            ";

            sqlx::query(upsert_query)
                .bind(data.token_chain_id.clone())
                .bind(data.token_address.clone())
                .bind(data.token_id.clone())
                .bind(block_timestamp as i64)
                .bind(block_timestamp as i64)
                .bind(data.offerer.clone())
                .bind(data.quantity.clone())
                .bind(data.start_amount.clone())
                .bind(data.end_amount.clone())
                .bind(data.start_date as i64)
                .bind(data.end_date as i64)
                .bind(data.broker_id.clone())
                .bind(data.order_hash.clone())
                .bind(data.currency_address.clone())
                .bind(data.currency_chain_id.clone())
                .bind(OrderStatus::Placed.to_string())
                .execute(&client.pool)
                .await?;
        }

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
                end_amount: Some(data.end_amount.clone()),
                canceled_reason: None,
                start_date: Some(data.start_date as i64),
                end_date: Some(data.end_date as i64),
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

        if let Some(token_data) =
            Self::get_token_data_by_order_hash(client, &data.order_hash).await?
        {
            Self::insert_event_history(
                client,
                &EventHistoryData {
                    token_id: token_data.token_id.clone(),
                    token_address: token_data.token_address.clone(),
                    event_type: token_data.order_type.clone().into(),
                    event_timestamp: block_timestamp as i64,
                    order_status: OrderStatus::Cancelled,
                    canceled_reason: data.reason.clone().into(),
                    new_owner: None,
                    amount: None,
                    previous_owner: None,
                    end_amount: None,
                    start_date: None,
                    end_date: None,
                },
            )
            .await?;

            Self::update_token_status(
                client,
                &token_data.token_address,
                &token_data.token_id,
                OrderStatus::Cancelled,
            )
            .await?;

            Self::clear_token_data_if_listing(
                client,
                &token_data.token_address,
                &token_data.token_id,
                token_data.order_type.as_str(),
            )
            .await?;
        }

        Self::update_offer_status(client, &data.order_hash, OrderStatus::Cancelled).await?;

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

        if let Some(token_data) =
            Self::get_token_data_by_order_hash(client, &data.order_hash).await?
        {
            Self::insert_event_history(
                client,
                &EventHistoryData {
                    token_id: token_data.token_id.clone(),
                    token_address: token_data.token_address.clone(),
                    event_type: token_data.order_type.clone().into(),
                    order_status: OrderStatus::Fulfilled,
                    event_timestamp: block_timestamp as i64,
                    canceled_reason: None,
                    new_owner: None,
                    amount: None,
                    previous_owner: None,
                    end_amount: None,
                    start_date: None,
                    end_date: None,
                },
            )
            .await?;

            Self::update_token_status(
                client,
                &token_data.token_address,
                &token_data.token_id,
                OrderStatus::Fulfilled,
            )
            .await?;
        }

        Self::update_offer_status(client, &data.order_hash, OrderStatus::Fulfilled).await?;

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

        if let Some(token_data) =
            Self::get_token_data_by_order_hash(client, &data.order_hash).await?
        {
            let mut new_owner = None;
            let mut previous_owner = None;
            match token_data.order_type.clone().into() {
                EventType::Offer | EventType::Auction | EventType::CollectionOffer => {
                    new_owner = Some(token_data.offerer);

                    Self::update_offer_status(client, &data.order_hash, OrderStatus::Executed)
                        .await?;

                    let params = OfferExecutedInfo {
                        block_timestamp,
                        token_address: token_data.token_address.clone(),
                        token_id: token_data.token_id.clone(),
                        new_owner: new_owner.clone().unwrap(),
                        price: token_data.start_amount.clone(),
                        order_hash: token_data.order_hash.clone(),
                        currency_chain_id: token_data.currency_chain_id.clone(),
                        currency_address: token_data.currency_address.clone(),
                    };

                    Self::update_owner_price_on_offer_executed(client, &params).await?;
                    previous_owner = Some(
                        Self::get_current_owner(
                            client,
                            &token_data.token_address,
                            &token_data.token_id,
                        )
                        .await?,
                    );
                }
                EventType::Listing => {
                    Self::update_token_on_listing_executed(
                        client,
                        &token_data.token_address,
                        &token_data.token_id,
                        block_timestamp as i64,
                        &token_data.start_amount,
                    )
                    .await?;
                }
            }

            Self::insert_event_history(
                client,
                &EventHistoryData {
                    token_id: token_data.token_id.clone(),
                    token_address: token_data.token_address.clone(),
                    event_type: token_data.order_type.clone().into(),
                    order_status: OrderStatus::Executed,
                    event_timestamp: block_timestamp as i64,
                    canceled_reason: None,
                    new_owner,
                    amount: None,
                    previous_owner,
                    end_amount: None,
                    start_date: None,
                    end_date: None,
                },
            )
            .await?;

            Self::update_token_status(
                client,
                &token_data.token_address,
                &token_data.token_id,
                OrderStatus::Executed,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn status_back_to_open(
        client: &SqlxCtx,
        block_id: u64,
        block_timestamp: u64,
        data: &RollbackStatusData,
    ) -> Result<(), ProviderError> {
        let mut string_reason = String::new();
        if let Some(first_char) = data.reason.chars().next() {
            let reason = first_char as u32;
            if let Some(status) = RollbackStatus::from_code(reason) {
                string_reason = status.to_string();
            }
        }

        if let Some(token_data) =
            Self::get_token_data_by_order_hash(client, &data.order_hash).await?
        {
            // we rollback to placed status
            Self::update_order_status(
                client,
                block_id,
                block_timestamp,
                &data.order_hash,
                OrderStatus::Placed,
            )
            .await?;

            Self::insert_event_history(
                client,
                &EventHistoryData {
                    token_id: token_data.token_id.clone(),
                    token_address: token_data.token_address.clone(),
                    event_type: token_data.order_type.clone().into(),
                    event_timestamp: block_timestamp as i64,
                    order_status: OrderStatus::Placed,
                    canceled_reason: Some(string_reason),
                    new_owner: None,
                    amount: None,
                    previous_owner: None,
                    end_amount: None,
                    start_date: None,
                    end_date: None,
                },
            )
            .await?;
        }
        Ok(())
    }
}

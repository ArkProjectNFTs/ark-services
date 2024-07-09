use crate::providers::{ProviderError, SqlxCtxPg};
use arkproject::diri::storage::types::{
    CancelledData, ExecutedData, FulfilledData, PlacedData, RollbackStatusData,
};
use async_std::stream::StreamExt;
use num_bigint::BigInt;
use num_traits::Num;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sqlx::types::BigDecimal;
use sqlx::Row;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, trace};

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

#[derive(Debug, Copy, Clone, PartialEq)]
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
    Fulfill,
    Executed,
    Cancelled,
    Mint,
    Burn,
    Transfer,
    Rollback,
}

impl From<String> for EventType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Listing" => EventType::Listing,
            "Auction" => EventType::Auction,
            "Offer" => EventType::Offer,
            "CollectionOffer" => EventType::CollectionOffer,
            "Fulfill" => EventType::Fulfill,
            "Executed" => EventType::Executed,
            "Cancelled" => EventType::Cancelled,
            "Mint" => EventType::Mint,
            "Burn" => EventType::Burn,
            "Transfer" => EventType::Transfer,
            "Rollback" => EventType::Rollback,
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
            "Fulfill" => Ok(EventType::Fulfill),
            "Executed" => Ok(EventType::Executed),
            "Cancelled" => Ok(EventType::Cancelled),
            "Mint" => Ok(EventType::Mint),
            "Burn" => Ok(EventType::Burn),
            "Transfer" => Ok(EventType::Transfer),
            "Rollback" => Ok(EventType::Rollback),
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
            EventType::Fulfill => "Fulfill",
            EventType::Executed => "Executed",
            EventType::Cancelled => "Cancelled",
            EventType::Mint => "Mint",
            EventType::Burn => "Burn",
            EventType::Transfer => "Transfer",
            EventType::Rollback => "Rollback",
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
                EventType::Fulfill => "Fulfill",
                EventType::Executed => "Executed",
                EventType::Cancelled => "Cancelled",
                EventType::Mint => "Mint",
                EventType::Burn => "Burn",
                EventType::Transfer => "Transfer",
                EventType::Rollback => "Rollback",
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
    order_hash: String,
    token_id: String,
    token_id_hex: String,
    contract_address: String,
    chain_id: String,
    event_type: EventType,
    block_timestamp: i64,
    from_address: Option<String>,
    to_address: Option<String>,
    amount: Option<String>,
    canceled_reason: Option<String>,
}

pub struct OfferData {
    order_hash: String,
    token_id: String,
    contract_address: String,
    chain_id: String,
    timestamp: i64,
    offer_maker: String,
    offer_amount: String,
    quantity: String,
    currency_chain_id: String,
    currency_address: String,
    status: String,
    start_date: i64,
    end_date: i64,
}

pub struct OfferExecutedInfo {
    block_timestamp: u64,
    contract_address: String,
    token_id: String,
    to_address: String,
    price: String,
    order_hash: String,
    currency_chain_id: String,
    currency_address: String,
}

#[derive(Debug)]
pub struct TokenData {
    token_id: String,
    token_id_hex: String,
    contract_address: String,
    chain_id: String,
    listing_start_amount: Option<String>,
    currency_chain_id: Option<String>,
}

impl OrderProvider {
    async fn clear_tokens_cache(
        redis_conn: Arc<Mutex<MultiplexedConnection>>,
        contract_address: &str,
    ) -> redis::RedisResult<()> {
        // Create a pattern for matching keys
        let pattern = format!("*{}_*", contract_address);

        // Collect keys matching the pattern
        let mut cmd = redis::cmd("SCAN");
        cmd.cursor_arg(0);
        cmd.arg("MATCH").arg(pattern);
        let mut conn = redis_conn.lock().await;
        let mut keys: Vec<String> = vec![];
        {
            let mut iter = cmd.iter_async::<_>(&mut *conn).await?;
            while let Some(key) = iter.next().await {
                keys.push(key);
            }
        }

        // Delete keys and log the results
        if !keys.is_empty() {
            conn.del(keys.clone()).await?;
        }

        Ok(())
    }

    async fn token_exists(
        client: &SqlxCtxPg,
        contract_address: &str,
        token_id: &str,
        chain_id: &str,
    ) -> Result<bool, ProviderError> {
        let query = "
            SELECT CASE
                WHEN EXISTS (
                    SELECT 1
                    FROM token
                    WHERE contract_address = $1 AND token_id = $2 AND chain_id = $3
                )
                THEN 1
                ELSE 0
            END;
        ";
        let exists: i32 = sqlx::query_scalar(query)
            .bind(contract_address)
            .bind(token_id.to_string())
            .bind(chain_id.to_string())
            .fetch_one(&client.pool)
            .await?;
        Ok(exists != 0)
    }

    async fn order_hash_exists_in_token(
        client: &SqlxCtxPg,
        order_hash: &str,
    ) -> Result<bool, ProviderError> {
        let query = "
            SELECT CASE
                WHEN EXISTS (
                    SELECT 1
                    FROM token
                    WHERE listing_orderhash = $1
                )
                THEN 1
                ELSE 0
            END;
        ";
        let exists: i32 = sqlx::query_scalar(query)
            .bind(order_hash)
            .fetch_one(&client.pool)
            .await?;
        Ok(exists != 0)
    }

    pub async fn get_contract(
        client: &SqlxCtxPg,
        contract_address: &str,
        chain_id: &str,
    ) -> Result<Option<String>, ProviderError> {
        let query = "
            SELECT contract_address
            FROM contract
            WHERE contract_address = $1 AND chain_id = $2;
        ";
        let result = sqlx::query(query)
            .bind(contract_address)
            .bind(chain_id)
            .fetch_optional(&client.pool)
            .await?;

        Ok(result.map(|row| row.get("contract_address")))
    }

    pub async fn get_or_create_contract(
        client: &SqlxCtxPg,
        contract_address: &str,
        chain_id: &str,
        block_timestamp: u64,
    ) -> Result<String, ProviderError> {
        match Self::get_contract(client, contract_address, chain_id).await? {
            Some(contract_address) => Ok(contract_address),
            None => {
                let insert_query = "
                        INSERT INTO contract (contract_address, chain_id, updated_timestamp, contract_type, deployed_timestamp)
                        VALUES ($1, $2, EXTRACT(epoch FROM now())::bigint, $3, $4)
                        RETURNING contract_address;
                    ";
                let result = sqlx::query(insert_query)
                    .bind(contract_address)
                    .bind(chain_id)
                    .bind("ERC721".to_string())
                    .bind(block_timestamp as i64)
                    .fetch_one(&client.pool)
                    .await?;
                Ok(result.get::<String, _>("contract_address"))
            }
        }
    }

    pub async fn get_offer_data_by_order_hash(
        client: &SqlxCtxPg,
        order_hash: &str,
    ) -> Result<Option<OfferData>, sqlx::Error> {
        let query = "
                SELECT  order_hash,
                        offer_timestamp,
                        offer_quantity,
                        status,
                        token_id,
                        contract_address,
                        chain_id,
                        offer_maker,
                        offer_amount,
                        currency_chain_id,
                        currency_address,
                        start_date,
                        end_date
                FROM token_offer
                WHERE order_hash = $1;
            ";

        if let Some((
            order_hash,
            timestamp,
            quantity,
            status,
            token_id,
            contract_address,
            chain_id,
            offer_maker,
            offer_amount,
            currency_chain_id,
            currency_address,
            start_date,
            end_date,
        )) = sqlx::query_as::<
            _,
            (
                String,
                i64,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                i64,
                i64,
            ),
        >(query)
        .bind(order_hash)
        .fetch_optional(&client.pool)
        .await?
        {
            Ok(Some(OfferData {
                order_hash,
                timestamp,
                quantity,
                status,
                token_id,
                contract_address,
                chain_id,
                offer_maker,
                offer_amount,
                currency_chain_id,
                currency_address,
                start_date,
                end_date,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_token_data_by_order_hash(
        client: &SqlxCtxPg,
        order_hash: &str,
    ) -> Result<Option<TokenData>, sqlx::Error> {
        let query = "
            SELECT token_id, token_id_hex, contract_address, chain_id, COALESCE(listing_start_amount, ''), currency_chain_id
            FROM token
            WHERE listing_orderhash = $1;
        ";

        if let Some((
            token_id,
            token_id_hex,
            contract_address,
            chain_id,
            listing_start_amount,
            currency_chain_id,
        )) = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
            ),
        >(query)
        .bind(order_hash)
        .fetch_optional(&client.pool)
        .await?
        {
            Ok(Some(TokenData {
                token_id,
                contract_address,
                token_id_hex,
                chain_id,
                listing_start_amount,
                currency_chain_id,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_current_owner(
        client: &SqlxCtxPg,
        contract_address: &String,
        token_id: &str,
        chain_id: &str,
    ) -> Result<Option<String>, ProviderError> {
        let query = "
            SELECT current_owner
            FROM token
            WHERE contract_address = $1 AND token_id = $2 AND chain_id = $3;
        ";
        let result = sqlx::query(query)
            .bind(contract_address)
            .bind(token_id)
            .bind(chain_id)
            .fetch_one(&client.pool)
            .await?;

        let current_owner: Option<String> = result.try_get::<String, _>("current_owner").ok();

        Ok(current_owner)
    }

    pub async fn get_fulfiller_address_from_event(
        client: &SqlxCtxPg,
        contract_address: &String,
        token_id: &str,
        chain_id: &str,
    ) -> Result<Option<String>, ProviderError> {
        let query = "
            SELECT from_address
            FROM token_event
            WHERE contract_address = $1 AND token_id = $2 AND chain_id = $3 and event_type = $4;
        ";
        let result = sqlx::query(query)
            .bind(contract_address)
            .bind(token_id)
            .bind(chain_id)
            .bind(EventType::Fulfill.to_string())
            .fetch_optional(&client.pool)
            .await?;

        match result {
            Some(row) => {
                let fulfiller: Option<String> = row.try_get::<String, _>("from_address").ok();
                Ok(fulfiller)
            }
            None => Ok(None),
        }
    }

    pub async fn get_token_data_by_id(
        client: &SqlxCtxPg,
        contract_address: &String,
        token_id: &str,
        chain_id: &str,
    ) -> Result<Option<TokenData>, sqlx::Error> {
        let query = "
            SELECT token_id, token_id_hex, contract_address, chain_id, COALESCE(listing_start_amount, ''), currency_chain_id
            FROM token
            WHERE contract_address = $1 AND token_id = $2 AND chain_id = $3;
        ";

        if let Some((
            token_id,
            token_id_hex,
            contract_address,
            chain_id,
            listing_start_amount,
            currency_chain_id,
        )) = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
            ),
        >(query)
        .bind(contract_address)
        .bind(token_id)
        .bind(chain_id)
        .fetch_optional(&client.pool)
        .await?
        {
            Ok(Some(TokenData {
                token_id,
                contract_address,
                token_id_hex,
                chain_id,
                listing_start_amount,
                currency_chain_id,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_token_status(
        client: &SqlxCtxPg,
        contract_address: &String,
        token_id: &str,
        status: OrderStatus,
    ) -> Result<(), ProviderError> {
        // if status is fulfilled, then buy_in_progress should be set to true
        let buy_in_progress = status == OrderStatus::Fulfilled;

        let query = "
        UPDATE token
        SET
            status = $3,
            buy_in_progress = $4
        WHERE contract_address = $1 AND token_id = $2;
        ";

        sqlx::query(query)
            .bind(contract_address)
            .bind(token_id)
            .bind(status.to_string())
            .bind(buy_in_progress)
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    pub async fn update_offer_status(
        client: &SqlxCtxPg,
        order_hash: &str,
        status: OrderStatus,
    ) -> Result<(), ProviderError> {
        let query = "UPDATE token_offer SET status = $2 WHERE order_hash = $1;";

        sqlx::query(query)
            .bind(order_hash)
            .bind(status.to_string())
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    pub async fn clear_token_data_if_listing(
        client: &SqlxCtxPg,
        contract_address: &String,
        token_id: &str,
        order_hash: &str,
    ) -> Result<(), ProviderError> {
        let order_in_token = Self::order_hash_exists_in_token(client, order_hash).await?;
        if order_in_token {
            let query = "
            UPDATE token
            SET
                listing_start_amount = null,
                listing_end_amount = null,
                listing_start_date = null,
                listing_end_date = null
            WHERE contract_address = $1 AND token_id = $2;
            ";

            sqlx::query(query)
                .bind(contract_address)
                .bind(token_id)
                .execute(&client.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn update_token_data_on_status_executed(
        client: &SqlxCtxPg,
        info: &OfferExecutedInfo,
    ) -> Result<(), ProviderError> {
        let query = "
            UPDATE token
            SET
                current_owner = $3, updated_timestamp = $4,
                last_price = $5, listing_orderhash = $6,
                currency_chain_id = $7, currency_address = $8,
                listing_start_date = null, listing_end_date = null,
                listing_start_amount = null, listing_end_amount = null,
                top_bid_amount = null, top_bid_start_date = null, top_bid_end_date = null, top_bid_currency_address = null,
                top_bid_order_hash = null,
                held_timestamp = $9
            WHERE contract_address = $1 AND token_id = $2;
        ";

        sqlx::query(query)
            .bind(&info.contract_address)
            .bind(&info.token_id)
            .bind(&info.to_address)
            .bind(info.block_timestamp as i64)
            .bind(&info.price)
            .bind(&info.order_hash.to_string())
            .bind(&info.currency_chain_id)
            .bind(&info.currency_address)
            .bind(info.block_timestamp as i64)
            .execute(&client.pool)
            .await?;

        // remove offers belonging to old owner
        let delete_query = "DELETE FROM token_offer WHERE offer_maker = $1 AND contract_address = $2 AND token_id = $3";
        sqlx::query(delete_query)
            .bind(&info.to_address)
            .bind(&info.contract_address)
            .bind(&info.token_id)
            .execute(&client.pool)
            .await?;

        let now = chrono::Utc::now().timestamp();
        let select_query = "
            SELECT hex_to_decimal(offer_amount), currency_address, start_date, end_date, order_hash
            FROM token_offer
            WHERE contract_address = $1 AND token_id = $2 AND end_date >= $3
            ORDER BY offer_amount DESC
            LIMIT 1
        ";
        let best_offer: Option<(BigDecimal, String, i64, i64, String)> =
            sqlx::query_as(select_query)
                .bind(&info.contract_address)
                .bind(&info.token_id)
                .bind(now)
                .fetch_optional(&client.pool)
                .await?;

        if let Some((offer_amount, currency_address, start_date, end_date, top_bid_order_hash)) =
            best_offer
        {
            let update_query = "
                UPDATE token
                SET top_bid_amount = $3, top_bid_start_date = $4, top_bid_end_date = $5, top_bid_currency_address = $6, top_bid_order_hash = $7
                WHERE contract_address = $1 AND token_id = $2
            ";
            sqlx::query(update_query)
                .bind(&info.contract_address)
                .bind(&info.token_id)
                .bind(offer_amount)
                .bind(start_date)
                .bind(end_date)
                .bind(currency_address)
                .bind(top_bid_order_hash)
                .execute(&client.pool)
                .await?;
        }

        Ok(())
    }

    async fn insert_event_history(
        client: &SqlxCtxPg,
        event_data: &EventHistoryData,
    ) -> Result<(), ProviderError> {
        if !Self::token_exists(
            client,
            &event_data.contract_address,
            &event_data.token_id,
            &event_data.chain_id,
        )
        .await?
        {
            return Err(ProviderError::from("Token does not exist"));
        }

        let token_event_id = format!("{}_{}", &event_data.order_hash, event_data.block_timestamp);

        let q = "
            INSERT INTO token_event (token_event_id, order_hash, token_id, token_id_hex, contract_address, chain_id, event_type, block_timestamp, from_address, to_address, amount, canceled_reason)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);
        ";

        let _r = sqlx::query(q)
            .bind(&token_event_id)
            .bind(&event_data.order_hash)
            .bind(&event_data.token_id)
            .bind(&event_data.token_id_hex)
            .bind(&event_data.contract_address)
            .bind(&event_data.chain_id)
            .bind(event_data.event_type.to_string())
            .bind(event_data.block_timestamp)
            .bind(&event_data.from_address.clone().unwrap_or_default())
            .bind(&event_data.to_address.clone().unwrap_or_default())
            .bind(&event_data.amount.clone().unwrap_or_default())
            .bind(&event_data.canceled_reason.clone().unwrap_or_default())
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    async fn offer_exists(
        client: &SqlxCtxPg,
        order_hash: &str,
        offer_timestamp: i64,
    ) -> Result<bool, ProviderError> {
        let query = "
            SELECT CASE
                WHEN EXISTS (
                    SELECT 1
                    FROM token_offer
                    WHERE order_hash = $1 AND offer_timestamp = $2
                )
                THEN 1
                ELSE 0
            END;
        ";
        let exists: i32 = sqlx::query_scalar(query)
            .bind(order_hash)
            .bind(offer_timestamp)
            .fetch_one(&client.pool)
            .await?;
        Ok(exists != 0)
    }

    async fn insert_offers(
        client: &SqlxCtxPg,
        offer_data: &OfferData,
    ) -> Result<(), ProviderError> {
        if Self::offer_exists(client, &offer_data.order_hash, offer_data.timestamp).await? {
            trace!("Offer already exists in database.");
            return Ok(());
        }

        if !Self::token_exists(
            client,
            &offer_data.contract_address,
            &offer_data.token_id,
            &offer_data.chain_id,
        )
        .await?
        {
            return Err(ProviderError::from("Token does not exist"));
        }

        // Check if topbid_amount is filled in token
        let topbid_query = "
           SELECT COALESCE(top_bid_amount, 0)
           FROM token
           WHERE contract_address = $1 AND token_id = $2 AND chain_id = $3;
       ";

        let topbid_amount: Option<BigDecimal> = sqlx::query_scalar(topbid_query)
            .bind(&offer_data.contract_address)
            .bind(&offer_data.token_id)
            .bind(&offer_data.chain_id)
            .fetch_optional(&client.pool)
            .await?;

        // If topbid_amount is filled and the offer is better, update topbid fields
        if let Some(topbid_amount) = topbid_amount {
            let offer_amount_hex = offer_data.offer_amount.trim_start_matches("0x");
            let offer_amount_bigint =
                BigInt::from_str_radix(offer_amount_hex, 16).unwrap_or_else(|_| BigInt::from(0));
            let offer_amount = BigDecimal::from(offer_amount_bigint);

            if offer_amount > topbid_amount {
                let update_query = "
                    UPDATE token
                    SET top_bid_amount = $4, top_bid_start_date = $5, top_bid_end_date = $6, top_bid_currency_address = $7, top_bid_order_hash = $8
                    WHERE contract_address = $1 AND token_id = $2 AND chain_id = $3;
                ";
                let result = sqlx::query(update_query)
                    .bind(&offer_data.contract_address)
                    .bind(&offer_data.token_id)
                    .bind(&offer_data.chain_id)
                    .bind(offer_amount)
                    .bind(offer_data.start_date)
                    .bind(offer_data.end_date)
                    .bind(&offer_data.currency_address)
                    .bind(&offer_data.order_hash)
                    .execute(&client.pool)
                    .await;

                match result {
                    Ok(_) => trace!("Update query executed successfully."),
                    Err(e) => error!("Error executing update query: {:?}", e),
                }
            }
        }

        let insert_query = "
            INSERT INTO token_offer
            (contract_address, token_id, chain_id, offer_maker, offer_amount, offer_quantity, offer_timestamp, order_hash, currency_chain_id, currency_address, status, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13);
        ";

        sqlx::query(insert_query)
            .bind(&offer_data.contract_address)
            .bind(&offer_data.token_id)
            .bind(&offer_data.chain_id)
            .bind(&offer_data.offer_maker)
            .bind(&offer_data.offer_amount)
            .bind(&offer_data.quantity)
            .bind(offer_data.timestamp)
            .bind(&offer_data.order_hash)
            .bind(&offer_data.currency_chain_id)
            .bind(&offer_data.currency_address)
            .bind(&offer_data.status)
            .bind(offer_data.start_date)
            .bind(offer_data.end_date)
            .execute(&client.pool)
            .await?;

        Ok(())
    }

    pub async fn register_placed(
        client: &SqlxCtxPg,
        redis_conn: Arc<Mutex<MultiplexedConnection>>,
        _block_id: u64,
        block_timestamp: u64,
        data: &PlacedData,
    ) -> Result<(), ProviderError> {
        trace!("Registering placed order {:?}", data);
        let token_id = match data.token_id {
            Some(ref token_id_hex) => {
                let cleaned_token_id = token_id_hex.trim_start_matches("0x");
                match BigInt::from_str_radix(cleaned_token_id, 16) {
                    Ok(token_id) => token_id.to_string(),
                    Err(e) => {
                        error!("Failed to parse token id: {}", e);
                        return Err(ProviderError::from("Failed to parse token id"));
                    }
                }
            }
            None => return Err(ProviderError::from("Missing token id")),
        };

        let event_type = EventType::from_str(&data.order_type).map_err(ProviderError::from)?;
        let contract_address = Self::get_or_create_contract(
            client,
            &data.token_address,
            &data.token_chain_id,
            block_timestamp,
        )
        .await?;

        match Self::clear_tokens_cache(redis_conn.clone(), &contract_address).await {
            Ok(_) => {}
            Err(e) => {
                println!("Error when deleting cache : {}", e);
            }
        }

        if event_type == EventType::Offer || event_type == EventType::CollectionOffer {
            // create token without listing information
            let upsert_query = "
                INSERT INTO token (contract_address, token_id, token_id_hex, chain_id, updated_timestamp, listing_orderhash, block_timestamp, status)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (contract_address, token_id, chain_id)
                DO NOTHING;
            ";

            sqlx::query(upsert_query)
                .bind(contract_address.clone())
                .bind(token_id.clone())
                .bind(data.token_id.clone())
                .bind(data.token_chain_id.clone())
                .bind(block_timestamp as i64)
                .bind(block_timestamp as i64)
                .bind(block_timestamp as i64)
                .bind(OrderStatus::Placed.to_string())
                .execute(&client.pool)
                .await?;

            Self::insert_offers(
                client,
                &OfferData {
                    token_id: token_id.clone(),
                    contract_address: contract_address.clone(),
                    chain_id: data.token_chain_id.clone(),
                    timestamp: block_timestamp as i64,
                    offer_maker: data.offerer.clone(),
                    offer_amount: data.start_amount.clone(),
                    quantity: data.quantity.clone(),
                    order_hash: data.order_hash.clone(),
                    currency_chain_id: data.currency_chain_id.clone(),
                    currency_address: data.currency_address.clone(),
                    status: OrderStatus::Placed.to_string(),
                    start_date: data.start_date as i64,
                    end_date: data.end_date as i64,
                },
            )
            .await?;
        } else {
            // create token with listing information
            let upsert_query = "
                INSERT INTO token (
                    contract_address,
                    token_id,
                    chain_id,
                    token_id_hex,
                    listing_timestamp,
                    updated_timestamp,
                    held_timestamp,
                    current_owner,
                    quantity,
                    listing_start_amount,
                    listing_end_amount,
                    listing_start_date,
                    listing_end_date,
                    listing_broker_id,
                    listing_orderhash,
                    listing_currency_address,
                    listing_currency_chain_id,
                    block_timestamp,
                    status,
                    listing_type)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                ON CONFLICT (token_id, contract_address, chain_id) DO UPDATE SET
                current_owner = EXCLUDED.current_owner,
                token_id_hex = EXCLUDED.token_id_hex,
                listing_timestamp = EXCLUDED.listing_timestamp,
                listing_start_amount = EXCLUDED.listing_start_amount,
                listing_end_amount = EXCLUDED.listing_end_amount,
                listing_start_date = EXCLUDED.listing_start_date,
                listing_end_date = EXCLUDED.listing_end_date,
                listing_broker_id = EXCLUDED.listing_broker_id,
                listing_orderhash = EXCLUDED.listing_orderhash,
                status = EXCLUDED.status,
                updated_timestamp = EXCLUDED.updated_timestamp,
                listing_type = EXCLUDED.listing_type;
            ";

            sqlx::query(upsert_query)
                .bind(contract_address.clone())
                .bind(token_id.clone())
                .bind(data.token_chain_id.clone())
                .bind(data.token_id.clone())
                .bind(block_timestamp as i64)
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
                .bind(block_timestamp as i64)
                .bind(OrderStatus::Placed.to_string())
                .bind(event_type.to_string())
                .execute(&client.pool)
                .await?;
        }

        if let Some(token_id_hex) = data.token_id.clone() {
            Self::insert_event_history(
                client,
                &EventHistoryData {
                    order_hash: data.order_hash.clone(),
                    token_id: token_id.clone(),
                    token_id_hex,
                    contract_address: contract_address.clone(),
                    chain_id: data.token_chain_id.clone(),
                    event_type,
                    block_timestamp: block_timestamp as i64,
                    from_address: None,
                    to_address: Some(data.offerer.clone()),
                    amount: Some(data.start_amount.clone()),
                    canceled_reason: None,
                },
            )
            .await?;
        }

        Ok(())
    }

    pub async fn register_cancelled(
        client: &SqlxCtxPg,
        redis_conn: Arc<Mutex<MultiplexedConnection>>,
        _block_id: u64,
        block_timestamp: u64,
        data: &CancelledData,
    ) -> Result<(), ProviderError> {
        trace!("Registering cancelled order {:?}", data);

        /* @TODO: maybe we have to check in offer ? */
        if let Some(token_data) =
            Self::get_token_data_by_order_hash(client, &data.order_hash).await?
        {
            match Self::clear_tokens_cache(redis_conn.clone(), &token_data.contract_address).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error when deleting cache : {}", e);
                }
            }

            let token_id = match BigInt::from_str(&token_data.token_id) {
                Ok(token_id) => token_id.to_string(),
                Err(e) => {
                    error!("Failed to parse token id: {}", e);
                    return Err(ProviderError::from("Failed to parse token id"));
                }
            };

            Self::insert_event_history(
                client,
                &EventHistoryData {
                    order_hash: data.order_hash.clone(),
                    token_id: token_id.clone(),
                    token_id_hex: token_data.token_id_hex.clone(),
                    contract_address: token_data.contract_address.clone(),
                    chain_id: token_data.chain_id.clone(),
                    event_type: EventType::Cancelled,
                    block_timestamp: block_timestamp as i64,
                    canceled_reason: data.reason.clone().into(),
                    to_address: None,
                    amount: None,
                    from_address: None,
                },
            )
            .await?;

            Self::update_token_status(
                client,
                &token_data.contract_address,
                &token_data.token_id,
                OrderStatus::Cancelled,
            )
            .await?;

            Self::clear_token_data_if_listing(
                client,
                &token_data.contract_address,
                &token_data.token_id,
                &data.order_hash,
            )
            .await?;
        }

        Self::update_offer_status(client, &data.order_hash, OrderStatus::Cancelled).await?;

        Ok(())
    }

    pub async fn register_fulfilled(
        client: &SqlxCtxPg,
        redis_conn: Arc<Mutex<MultiplexedConnection>>,
        _block_id: u64,
        block_timestamp: u64,
        data: &FulfilledData,
    ) -> Result<(), ProviderError> {
        trace!("Registering fulfilled order {:?}", data);
        if let Some(token_data) =
            Self::get_token_data_by_order_hash(client, &data.order_hash).await?
        {
            let token_id = match BigInt::from_str(&token_data.token_id) {
                Ok(token_id) => token_id.to_string(),
                Err(e) => {
                    error!("Failed to parse token id: {}", e);
                    return Err(ProviderError::from("Failed to parse token id"));
                }
            };

            match Self::clear_tokens_cache(redis_conn.clone(), &token_data.contract_address).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error when deleting cache : {}", e);
                }
            }

            Self::insert_event_history(
                client,
                &EventHistoryData {
                    order_hash: data.order_hash.clone(),
                    token_id: token_id.clone(),
                    token_id_hex: token_data.token_id_hex.clone(),
                    contract_address: token_data.contract_address.clone(),
                    chain_id: token_data.chain_id.clone(),
                    event_type: EventType::Fulfill,
                    block_timestamp: block_timestamp as i64,
                    canceled_reason: None,
                    to_address: None,
                    amount: None,
                    from_address: Some(data.fulfiller.clone()),
                },
            )
            .await?;

            Self::update_token_status(
                client,
                &token_data.contract_address,
                &token_data.token_id,
                OrderStatus::Fulfilled,
            )
            .await?;

            Self::update_offer_status(client, &data.order_hash, OrderStatus::Fulfilled).await?;
        }
        Ok(())
    }

    pub async fn register_executed(
        client: &SqlxCtxPg,
        redis_conn: Arc<Mutex<MultiplexedConnection>>,
        _block_id: u64,
        block_timestamp: u64,
        data: &ExecutedData,
    ) -> Result<(), ProviderError> {
        trace!("Registering executed order {:?}", data);
        if let Some(offer_data) =
            Self::get_offer_data_by_order_hash(client, &data.order_hash).await?
        {
            match Self::clear_tokens_cache(redis_conn.clone(), &offer_data.contract_address).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error when deleting cache : {}", e);
                }
            }
            if let Some(token_data) = Self::get_token_data_by_id(
                client,
                &offer_data.contract_address,
                &offer_data.token_id,
                &offer_data.chain_id,
            )
            .await?
            {
                /* EventType::Offer | EventType::CollectionOffer */
                let to_address = Some(offer_data.offer_maker.clone());
                Self::update_offer_status(client, &data.order_hash, OrderStatus::Executed).await?;
                let from_address = Self::get_current_owner(
                    client,
                    &offer_data.contract_address,
                    &offer_data.token_id,
                    &offer_data.chain_id,
                )
                .await?;
                let params = OfferExecutedInfo {
                    block_timestamp,
                    contract_address: offer_data.contract_address.clone(),
                    token_id: offer_data.token_id.clone(),
                    to_address: offer_data.offer_maker.clone(),
                    price: offer_data.offer_amount.clone(),
                    order_hash: data.order_hash.clone(),
                    currency_chain_id: offer_data.currency_chain_id.clone(),
                    currency_address: offer_data.currency_address.clone(),
                };
                Self::update_token_data_on_status_executed(client, &params).await?;

                Self::insert_event_history(
                    client,
                    &EventHistoryData {
                        order_hash: data.order_hash.clone(),
                        token_id: offer_data.token_id.clone(),
                        token_id_hex: token_data.token_id_hex.clone(),
                        contract_address: offer_data.contract_address.clone(),
                        chain_id: offer_data.chain_id.clone(),
                        event_type: EventType::Executed,
                        block_timestamp: block_timestamp as i64,
                        canceled_reason: None,
                        to_address,
                        from_address,
                        amount: None,
                    },
                )
                .await?;
            }
        } else {
            // listing
            let order_in_token = Self::order_hash_exists_in_token(client, &data.order_hash).await?;
            if order_in_token {
                if let Some(token_data) =
                    Self::get_token_data_by_order_hash(client, &data.order_hash).await?
                {
                    match Self::clear_tokens_cache(redis_conn.clone(), &token_data.contract_address)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error when deleting cache : {}", e);
                        }
                    }

                    let fulfiller = Self::get_fulfiller_address_from_event(
                        client,
                        &token_data.contract_address,
                        &token_data.token_id,
                        &token_data.chain_id,
                    )
                    .await?;

                    /* EventType::Listing | EventType::Auction */
                    let params = OfferExecutedInfo {
                        block_timestamp,
                        contract_address: token_data.contract_address.clone(),
                        token_id: token_data.token_id.clone(),
                        to_address: fulfiller.clone().unwrap_or_default(),
                        price: token_data.listing_start_amount.clone().unwrap_or_default(),
                        order_hash: data.order_hash.clone(),
                        currency_chain_id: token_data.chain_id.clone(),
                        currency_address: token_data.currency_chain_id.clone().unwrap_or_default(),
                    };

                    Self::update_token_data_on_status_executed(client, &params).await?;
                    Self::insert_event_history(
                        client,
                        &EventHistoryData {
                            order_hash: data.order_hash.clone(),
                            block_timestamp: block_timestamp as i64,
                            token_id: token_data.token_id.clone(),
                            token_id_hex: token_data.token_id_hex.clone(),
                            contract_address: token_data.contract_address.clone(),
                            chain_id: token_data.chain_id,
                            event_type: EventType::Executed,
                            canceled_reason: None,
                            to_address: None,
                            amount: token_data.listing_start_amount,
                            from_address: None,
                        },
                    )
                    .await?;

                    Self::update_token_status(
                        client,
                        &token_data.contract_address,
                        &token_data.token_id,
                        OrderStatus::Executed,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn status_back_to_open(
        client: &SqlxCtxPg,
        _block_id: u64,
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
            Self::insert_event_history(
                client,
                &EventHistoryData {
                    order_hash: data.order_hash.clone(),
                    block_timestamp: block_timestamp as i64,
                    token_id: token_data.token_id.clone(),
                    token_id_hex: token_data.token_id_hex.clone(),
                    contract_address: token_data.contract_address,
                    chain_id: token_data.chain_id,
                    event_type: EventType::Rollback,
                    canceled_reason: Some(string_reason),
                    to_address: None,
                    amount: None,
                    from_address: None,
                },
            )
            .await?;
        }
        Ok(())
    }
}

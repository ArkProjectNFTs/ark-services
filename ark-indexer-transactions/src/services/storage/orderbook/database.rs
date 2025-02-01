use super::{
    constants::{
        sql_cancelled_reason_type, sql_order_event_type, sql_order_status, sql_order_type,
        sql_route_type,
    },
    OrderbookStorage,
};
use crate::{
    interfaces::orderbook::OrderTransactionInfo,
    services::storage::{
        database::DatabaseStorage,
        models::{currency::Currency, ExistingOrder, ExistingOrderType},
    },
};
use arkproject::orderbook::{
    self,
    events::{
        common::{u256_to_biguint, u256_to_hex},
        OrderCancelled, OrderExecuted, OrderFulfilled, OrderPlaced,
    },
    OrderType, RouteType,
};
use bigdecimal::BigDecimal;
use sqlx::{encode::IsNull, postgres::PgArgumentBuffer, Encode, Postgres};
use starknet_crypto::Felt;
use tracing::{debug, info, warn};

enum OrderEventType {
    Placed,
    Cancelled,
    Fulfilled,
    Executed,
}

impl AsRef<str> for OrderEventType {
    fn as_ref(&self) -> &str {
        match self {
            OrderEventType::Placed => sql_order_event_type::PLACED,
            OrderEventType::Cancelled => sql_order_event_type::CANCELLED,
            OrderEventType::Fulfilled => sql_order_event_type::FULFILLED,
            OrderEventType::Executed => sql_order_event_type::EXECUTED,
        }
    }
}

impl sqlx::Type<Postgres> for OrderEventType {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name(sql_order_event_type::TYPE_NAME)
    }
}

impl Encode<'_, Postgres> for OrderEventType {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>> {
        <&str as Encode<Postgres>>::encode(self.as_ref(), buf)
    }
}

#[derive(Copy, Clone)]
enum OrderStatus {
    Open,
    Executed,
    Cancelled,
}

impl AsRef<str> for OrderStatus {
    fn as_ref(&self) -> &str {
        match self {
            OrderStatus::Open => sql_order_status::OPEN,
            OrderStatus::Executed => sql_order_status::EXECUTED,
            OrderStatus::Cancelled => sql_order_status::CANCELLED,
        }
    }
}

impl sqlx::Type<Postgres> for OrderStatus {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name(sql_order_status::TYPE_NAME)
    }
}

impl Encode<'_, Postgres> for OrderStatus {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as Encode<Postgres>>::encode(self.as_ref(), buf)
    }
}

struct RouteTypeWrapper(RouteType);

impl AsRef<str> for RouteTypeWrapper {
    fn as_ref(&self) -> &str {
        match self.0 {
            RouteType::Erc20ToErc721 => sql_route_type::ERC20_TO_ERC721,
            RouteType::Erc721ToErc20 => sql_route_type::ERC721_TO_ERC20,
            RouteType::Erc20ToErc1155 => sql_route_type::ERC20_TO_ERC1155,
            RouteType::Erc1155ToErc20 => sql_route_type::ERC1155_TO_ERC20,
        }
    }
}

impl sqlx::Type<Postgres> for RouteTypeWrapper {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name(sql_route_type::TYPE_NAME)
    }
}

impl Encode<'_, Postgres> for RouteTypeWrapper {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>> {
        <&str as Encode<Postgres>>::encode(self.as_ref(), buf)
    }
}

struct OrderTypeWrapper(OrderType);
impl AsRef<str> for OrderTypeWrapper {
    fn as_ref(&self) -> &str {
        match self.0 {
            OrderType::Listing => sql_order_type::LISTING,
            OrderType::Auction => sql_order_type::AUCTION,
            OrderType::Offer => sql_order_type::OFFER,
            OrderType::CollectionOffer => sql_order_type::COLLECTION_OFFER,
        }
    }
}

impl sqlx::Type<Postgres> for OrderTypeWrapper {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name(sql_order_type::TYPE_NAME)
    }
}

impl Encode<'_, Postgres> for OrderTypeWrapper {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>> {
        <&str as Encode<Postgres>>::encode(self.as_ref(), buf)
    }
}

enum CancelledReason {
    User,
    ByNewOrder,
    AssetFault,
    Ownership,
    Unknown,
}

impl AsRef<str> for CancelledReason {
    fn as_ref(&self) -> &str {
        match self {
            CancelledReason::User => sql_cancelled_reason_type::USER,
            CancelledReason::ByNewOrder => sql_cancelled_reason_type::BY_NEW_ORDER,
            CancelledReason::AssetFault => sql_cancelled_reason_type::ASSET_FAULT,
            CancelledReason::Ownership => sql_cancelled_reason_type::OWNERSHIP,
            CancelledReason::Unknown => sql_cancelled_reason_type::UNKNOWN,
        }
    }
}

impl From<Felt> for CancelledReason {
    fn from(value: Felt) -> Self {
        if value == orderbook::error::CANCELLED_USER {
            return CancelledReason::User;
        }
        if value == orderbook::error::CANCELLED_BY_NEW_ORDER {
            return CancelledReason::ByNewOrder;
        }
        if value == orderbook::error::CANCELLED_ASSET_FAULT {
            return CancelledReason::AssetFault;
        }
        if value == orderbook::error::CANCELLED_OWNERSHIP {
            return CancelledReason::Ownership;
        }
        CancelledReason::Unknown
    }
}

impl sqlx::Type<Postgres> for CancelledReason {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name(sql_cancelled_reason_type::TYPE_NAME)
    }
}

impl Encode<'_, Postgres> for CancelledReason {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>> {
        <&str as Encode<Postgres>>::encode(self.as_ref(), buf)
    }
}

impl DatabaseStorage {
    async fn remove_from_active_order(
        &self,
        order_hash: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            DELETE FROM active_orders WHERE order_hash = $1
        "#;
        sqlx::query(query)
            .bind(order_hash)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn update_order_status(
        &self,
        order_hash: String,
        order_status: OrderStatus,
        timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            UPDATE orders
                SET status = $2, updated_at = $3
            WHERE order_hash = $1
        "#;

        sqlx::query(query)
            .bind(order_hash.clone())
            .bind(order_status)
            .bind(timestamp as i64)
            .execute(self.pool())
            .await?;

        // remove from active orders if needed
        match order_status {
            OrderStatus::Open => (),
            OrderStatus::Executed | OrderStatus::Cancelled => {
                self.remove_from_active_order(order_hash).await?
            }
        };

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_orderbook_transaction_info(
        &self,
        transaction_info: &OrderTransactionInfo,
        order_hash: String,
        order_event_type: OrderEventType,
        cancelled_reason: Option<CancelledReason>,
        related_order_hash: Option<String>,
        fulfiller: Option<String>,
        from: Option<String>,
        to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
        INSERT INTO order_transaction_info (
            tx_hash, event_id, order_hash, 
            timestamp,
            event_type,
            cancelled_reason,
            related_order_hash,
            fulfiller,
            from_address, to_address
        ) VALUES (
            $1, $2, $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9, $10
        )
        "#;
        sqlx::query(query)
            .bind(&transaction_info.tx_hash)
            .bind(transaction_info.event_id as i64)
            .bind(order_hash)
            .bind(transaction_info.timestamp as i64)
            .bind(order_event_type)
            .bind(cancelled_reason)
            .bind(related_order_hash)
            .bind(fulfiller)
            .bind(from)
            .bind(to)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn get_currency_by_contract_address(
        &self,
        contract_address: &str,
    ) -> Result<Option<Currency>, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT 
                contract_address,
                chain_id,
                symbol,
                decimals,
                price_in_usd,
                price_in_eth,
                price_updated_at
            FROM currency
            WHERE contract_address = $1
        "#;

        let currency = sqlx::query_as::<_, Currency>(query)
            .bind(contract_address)
            .fetch_optional(self.pool())
            .await?;

        Ok(currency)
    }

    async fn handle_order_placed(
        &self,
        orderbook_transaction_info: &OrderTransactionInfo,
        order_placed: &OrderPlaced,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling order placed event");

        let query = r#"
            INSERT INTO orders (
                order_hash, created_at, route_type, order_type,
                currency_address, currency_chain_id, offerer, 
                token_chain_id, token_address, token_id, token_id_hex,
                quantity, start_amount, end_amount,
                start_date, end_date,
                broker_id,
                cancelled_order_hash,
                updated_at,
                status,
                start_amount_eth
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7,
                $8, $9, $10,
                $11, $12, $13,
                $14, $15,
                $16,
                $17,
                $18, 
                $19,
                $20,
                $21
            )
        "#;

        let order_hash = match order_placed {
            OrderPlaced::V1(order_placed) => {
                info!("Processing V1 order placed");

                let order_hash = order_placed.order_hash.to_fixed_hex_string();
                let order = &order_placed.order;

                let token_id_hex = order.token_id.map(|value| u256_to_hex(&value));

                let token_id = order
                    .token_id
                    .map(|value| u256_to_biguint(&value).to_string());

                let cancelled_order_hash = order_placed
                    .cancelled_order_hash
                    .map(|value| value.to_fixed_hex_string());
                let route_type = RouteTypeWrapper(RouteType::from(&order.route));
                let order_type = OrderTypeWrapper(OrderType::from(&order_placed.order_type));

                let currency_contract_address = &order.currency_address.0.to_fixed_hex_string();

                let start_amount_eth = if let Ok(Some(currency)) = self
                    .get_currency_by_contract_address(currency_contract_address)
                    .await
                {
                    let start_amount_u256 = starknet::core::types::U256::from_words(
                        order.start_amount.low,
                        order.start_amount.high,
                    );

                    let start_amount_decimal = start_amount_u256
                        .to_string()
                        .parse::<BigDecimal>()
                        .unwrap_or_else(|_| BigDecimal::from(0));
                    let value = start_amount_decimal * currency.price_in_eth;

                    value
                        .to_string()
                        .parse::<BigDecimal>()
                        .unwrap_or_else(|_| BigDecimal::from(0))
                } else {
                    info!("No currency info found, using 0 as ETH amount");
                    BigDecimal::from(0)
                };

                sqlx::query(query)
                    .bind(order_hash.clone())
                    .bind(orderbook_transaction_info.timestamp as i64)
                    .bind(route_type)
                    .bind(&order_type)
                    .bind(currency_contract_address)
                    .bind(order.currency_chain_id.to_fixed_hex_string())
                    .bind(order.offerer.0.to_fixed_hex_string())
                    .bind(order.token_chain_id.to_fixed_hex_string())
                    .bind(order.token_address.0.to_fixed_hex_string())
                    .bind(token_id.clone())
                    .bind(token_id_hex)
                    .bind(u256_to_hex(&order.quantity))
                    .bind(u256_to_hex(&order.start_amount))
                    .bind(u256_to_hex(&order.end_amount))
                    .bind(order.start_date as i64)
                    .bind(order.end_date as i64)
                    .bind(order.broker_id.0.to_fixed_hex_string())
                    .bind(cancelled_order_hash)
                    .bind(orderbook_transaction_info.timestamp as i64) // updated_at
                    .bind(OrderStatus::Open)
                    .bind(&start_amount_eth)
                    .execute(self.pool())
                    .await?;

                if let Some(token_id) = token_id {
                    let query = r#"
                    INSERT INTO token_event (
                        token_event_id, contract_address, chain_id, broker_id, order_hash, 
                        token_id, event_type, block_timestamp, transaction_hash, 
                        from_address, amount,
                        token_sub_event_id, currency_address, amount_eth
                    ) VALUES (
                        $1, $2, $3, $4, $5, 
                        $6, $7, $8, $9, 
                        $10, $11,
                        $12, $13, $14
                    )
                "#;

                    let token_event_id = format!(
                        "{}_{}",
                        orderbook_transaction_info.tx_hash, orderbook_transaction_info.event_id
                    );

                    info!(
                        "Inserting token event for order {} (token_event_id: {}, contract: {}, chain: {}, token: {}, broker_id: {})",
                        order_hash,
                        token_event_id,
                        order.token_address.0.to_fixed_hex_string(),
                        order.token_chain_id.to_hex_string(),
                        token_id.to_string(),
                        order.broker_id.0.to_fixed_hex_string()
                    );

                    sqlx::query(query)
                        .bind(token_event_id)
                        .bind(order.token_address.0.to_fixed_hex_string())
                        .bind(order.token_chain_id.to_hex_string())
                        .bind(order.broker_id.0.to_fixed_hex_string())
                        .bind(order_hash.clone())
                        .bind(token_id.clone())
                        .bind(&order_type)
                        .bind(orderbook_transaction_info.timestamp as i64)
                        .bind(orderbook_transaction_info.tx_hash.clone())
                        .bind(order.offerer.0.to_fixed_hex_string())
                        .bind(u256_to_hex(&order.start_amount))
                        .bind(orderbook_transaction_info.sub_event_id.clone())
                        .bind(currency_contract_address.clone())
                        .bind(&start_amount_eth)
                        .execute(self.pool())
                        .await?;
                }

                order_hash
            }
        };

        self.insert_orderbook_transaction_info(
            orderbook_transaction_info,
            order_hash,
            OrderEventType::Placed,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    async fn handle_order_cancelled(
        &self,
        transaction_info: &OrderTransactionInfo,
        order_cancelled: &OrderCancelled,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (order_hash, cancelled_reason) = match order_cancelled {
            OrderCancelled::V1(order_cancelled) => {
                let order_hash = order_cancelled.order_hash.to_fixed_hex_string();
                let cancelled_reason = Some(CancelledReason::from(order_cancelled.reason));
                (order_hash, cancelled_reason)
            }
        };

        let query = r#"
            SELECT token_address, token_id, broker_id, start_amount_eth, start_amount, order_type
            FROM public.orders
            WHERE order_hash = $1
            LIMIT 1
        "#;

        let existing_order = sqlx::query_as::<_, ExistingOrder>(query)
            .bind(&order_hash)
            .fetch_optional(self.pool())
            .await?;

        if let Some(existing_order) = existing_order {
            let token_event_id =
                format!("{}_{}", transaction_info.tx_hash, transaction_info.event_id);

            let query = r#"
                INSERT INTO token_event (
                    token_event_id, contract_address, token_id, chain_id, order_hash, event_type,
                    block_timestamp, transaction_hash, token_sub_event_id, canceled_reason
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
                )
            "#;

            let event_type = match existing_order.order_type {
                ExistingOrderType::Listing => "ListingCancelled",
                ExistingOrderType::Auction => "AuctionCancelled",
                ExistingOrderType::Offer => "OfferCancelled",
                ExistingOrderType::CollectionOffer => "OfferCancelled",
            };

            sqlx::query(query)
                .bind(token_event_id)
                .bind(existing_order.token_address)
                .bind(existing_order.token_id)
                .bind(&transaction_info.chain_id)
                .bind(order_hash.clone())
                .bind(event_type)
                .bind(transaction_info.timestamp as i64)
                .bind(transaction_info.tx_hash.clone())
                .bind(transaction_info.sub_event_id.clone())
                .bind(&cancelled_reason)
                .execute(self.pool())
                .await?;
        }

        self.update_order_status(
            order_hash.clone(),
            OrderStatus::Cancelled,
            transaction_info.timestamp,
        )
        .await?;

        self.insert_orderbook_transaction_info(
            transaction_info,
            order_hash,
            OrderEventType::Cancelled,
            cancelled_reason,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    async fn handle_order_fulfilled(
        &self,
        transaction_info: &OrderTransactionInfo,
        order_fulfilled: &OrderFulfilled,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (order_hash, fulfiller, related_order_hash) = match order_fulfilled {
            OrderFulfilled::V1(order_fulfilled) => {
                let order_hash = order_fulfilled.order_hash.to_fixed_hex_string();
                let fulfiller = order_fulfilled.fulfiller.0.to_fixed_hex_string();
                let related_order_hash = order_fulfilled
                    .related_order_hash
                    .map(|value| value.to_fixed_hex_string());

                (order_hash, fulfiller, related_order_hash)
            }
        };

        self.insert_orderbook_transaction_info(
            transaction_info,
            order_hash,
            OrderEventType::Fulfilled,
            None,
            related_order_hash,
            Some(fulfiller),
            None,
            None,
        )
        .await?;

        Ok(())
    }

    async fn handle_order_executed(
        &self,
        transaction_info: &OrderTransactionInfo,
        order_executed: &OrderExecuted,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (order_hash, from, to) = match order_executed {
            OrderExecuted::V0(order_executed) => {
                (order_executed.order_hash.to_fixed_hex_string(), None, None)
            }
            OrderExecuted::V1(order_executed) => (
                order_executed.order_hash.to_fixed_hex_string(),
                Some(order_executed.from.0.to_fixed_hex_string()),
                Some(order_executed.to.0.to_fixed_hex_string()),
            ),
            OrderExecuted::V2(order_executed) => (
                order_executed.order_hash.to_fixed_hex_string(),
                Some(order_executed.from.0.to_fixed_hex_string()),
                Some(order_executed.to.0.to_fixed_hex_string()),
            ),
        };

        debug!(order_hash = %order_hash, from = ?from, to = ?to, "Handling order executed");

        self.update_order_status(
            order_hash.clone(),
            OrderStatus::Executed,
            transaction_info.timestamp,
        )
        .await?;

        debug!("Updated order status to Executed");

        self.insert_orderbook_transaction_info(
            transaction_info,
            order_hash.clone(),
            OrderEventType::Executed,
            None,
            None,
            None,
            from.clone(),
            to.clone(),
        )
        .await?;

        debug!("Inserted orderbook transaction info");

        let query = r#"
            SELECT token_address, token_id, broker_id, start_amount_eth, start_amount, order_type, currency_address, currency_chain_id
            FROM orders
            WHERE order_hash = $1
            LIMIT 1
        "#;

        let existing_order = sqlx::query_as::<_, ExistingOrder>(query)
            .bind(&order_hash)
            .fetch_optional(self.pool())
            .await?;

        if let Some(existing_order) = existing_order {
            debug!(
                token_address = %existing_order.token_address,
                token_id = ?existing_order.token_id,
                "Found existing order"
            );

            let query = r#"
                    INSERT INTO token_event (
                        token_event_id, contract_address, chain_id, order_hash,
                        token_id, event_type, block_timestamp, transaction_hash,
                        from_address, to_address, amount,
                        token_sub_event_id, currency_address, amount_eth
                    ) VALUES (
                        $1, $2, $3, $4, $5,
                        $6, $7, $8, $9,
                        $10, $11,
                        $12, $13, $14
                    )
                "#;

            let token_event_id =
                format!("{}_{}", transaction_info.tx_hash, transaction_info.event_id);

            let amount = existing_order
                .start_amount
                .parse::<BigDecimal>()
                .unwrap_or_else(|_| BigDecimal::from(0));

            debug!(
                token_event_id = %token_event_id,
                amount = %amount,
                "Inserting token event"
            );

            sqlx::query(query)
                .bind(token_event_id)
                .bind(existing_order.token_address)
                .bind(transaction_info.chain_id.to_string())
                .bind(order_hash.clone())
                .bind(existing_order.token_id)
                .bind("Sale")
                .bind(transaction_info.timestamp as i64)
                .bind(transaction_info.tx_hash.clone())
                .bind(from)
                .bind(to)
                .bind(amount.to_string())
                .bind(transaction_info.sub_event_id.clone())
                .bind(existing_order.currency_address)
                .bind(existing_order.start_amount_eth)
                .execute(self.pool())
                .await?;

            debug!("Successfully inserted token event");
        } else {
            debug!(order_hash = %order_hash, "No existing order found");
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl OrderbookStorage for DatabaseStorage {
    async fn store_orderbook_transaction_info(
        &self,
        transaction_info: OrderTransactionInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match transaction_info.event {
            orderbook::Event::OrderPlaced(ref order_placed) => {
                debug!(event = "order_placed", tx_hash = %transaction_info.tx_hash);
                self.handle_order_placed(&transaction_info, order_placed)
                    .await?
            }
            orderbook::Event::OrderCancelled(ref order_cancelled) => {
                debug!(event = "order_cancelled", tx_hash = %transaction_info.tx_hash);
                self.handle_order_cancelled(&transaction_info, order_cancelled)
                    .await?
            }
            orderbook::Event::OrderExecuted(ref order_executed) => {
                debug!(event = "order_executed", tx_hash = %transaction_info.tx_hash);
                self.handle_order_executed(&transaction_info, order_executed)
                    .await?
            }
            orderbook::Event::OrderFulfilled(ref order_fulfilled) => {
                debug!(event = "order_fulfilled", tx_hash = %transaction_info.tx_hash);
                self.handle_order_fulfilled(&transaction_info, order_fulfilled)
                    .await?
            }
            _ => {
                warn!(event = ?transaction_info.event, tx_hash = %transaction_info.tx_hash, "Unsupported event");
            }
        };
        Ok(())
    }
}

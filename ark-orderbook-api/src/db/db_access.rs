use crate::models::token::{
    RawTokenData, TokenData, TokenHistory, TokenOffer, TokenWithHistory, TokenWithOffers,
};
use async_trait::async_trait;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
pub trait DatabaseAccess: Send + Sync {
    async fn get_token_data(&self, token_address: &str, token_id: &str)
        -> Result<TokenData, Error>;
    async fn get_token_by_collection_data(
        &self,
        token_address: &str,
    ) -> Result<Vec<TokenData>, Error>;
    async fn get_token_history_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenWithHistory, Error>;
    async fn get_token_offers_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenWithOffers, Error>;
    async fn get_tokens_by_owner_data(&self, owner: &str) -> Result<Vec<TokenData>, Error>;
    async fn delete_token_data(&self, token_address: &str, token_id: &str) -> Result<u64, Error>;
    async fn flush_all_data(&self) -> Result<u64, Error>;
    async fn delete_migrations(&self) -> Result<u64, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_token_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenData, Error> {
        let token_data = sqlx::query_as!(
            RawTokenData,
            "SELECT
                t.order_hash, t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.last_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                (
                    t.start_date IS NOT NULL AND t.end_date IS NOT NULL
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN t.start_date AND t.end_date
                    AND t.status != 'CANCELLED'
                ) AS is_listed,
                EXISTS(
                        SELECT 1
                        FROM orderbook_token_offers o
                        WHERE o.token_id = t.token_id
                        AND o.token_address = t.token_address
                        AND o.status not in ('CANCELLED', 'FULFILLED', 'EXECUTED')
                        AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                    ) AS has_offer,
                t.currency_chain_id, t.currency_address,
                (SELECT offer_amount FROM orderbook_token_offers WHERE token_id = t.token_id AND token_address = t.token_address AND status = 'PLACED' AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN start_date AND end_date ORDER BY offer_amount DESC LIMIT 1) AS top_bid_amount,
                (SELECT order_hash FROM orderbook_token_offers WHERE token_id = t.token_id AND token_address = t.token_address AND status = 'PLACED' AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN start_date AND end_date ORDER BY offer_amount DESC LIMIT 1) AS top_bid_order_hash,
                t.status,
                t.buy_in_progress
            FROM
                orderbook_token t
            LEFT JOIN
                orderbook_token_history th ON th.token_id = t.token_id AND th.token_address = t.token_address
            WHERE
                t.token_address = $1 AND t.token_id = $2
            GROUP BY
                t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.last_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,th.event_type, th.event_timestamp
            ORDER BY
                th.event_timestamp DESC;",
            token_address,
            token_id
        ).fetch_one(self).await?;

        let token: TokenData = TokenData::from(token_data);

        Ok(token)
    }

    async fn get_tokens_by_owner_data(&self, owner: &str) -> Result<Vec<TokenData>, Error> {
        let tokens_data = sqlx::query_as!(
            RawTokenData,
            "SELECT
                t.order_hash, t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.last_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                (
                    t.start_date IS NOT NULL AND t.end_date IS NOT NULL
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN t.start_date AND t.end_date
                    AND t.status != 'CANCELLED'
                ) AS is_listed,
                EXISTS(
                        SELECT 1
                        FROM orderbook_token_offers o
                        WHERE o.token_id = t.token_id
                        AND o.token_address = t.token_address
                        AND o.status not in ('CANCELLED', 'FULFILLED', 'EXECUTED')
                        AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                    ) AS has_offer,
                t.currency_chain_id, t.currency_address,
                (SELECT offer_amount FROM orderbook_token_offers WHERE token_id = t.token_id AND token_address = t.token_address AND status = 'PLACED' AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN start_date AND end_date ORDER BY offer_amount DESC LIMIT 1) AS top_bid_amount,
                (SELECT order_hash FROM orderbook_token_offers WHERE token_id = t.token_id AND token_address = t.token_address AND status = 'PLACED' AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN start_date AND end_date ORDER BY offer_amount DESC LIMIT 1) AS top_bid_order_hash,
                t.status,
                t.buy_in_progress
            FROM
                orderbook_token t
            LEFT JOIN
                orderbook_token_history th ON th.token_id = t.token_id AND th.token_address = t.token_address
            WHERE
                t.current_owner = $1
            GROUP BY
                t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.last_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,th.event_type, th.event_timestamp
            ORDER BY
                th.event_timestamp DESC;",
            owner
        ).fetch_all(self).await?;

        let tokens: Vec<TokenData> = tokens_data.into_iter().map(TokenData::from).collect();

        Ok(tokens)
    }

    async fn get_token_by_collection_data(
        &self,
        token_address: &str,
    ) -> Result<Vec<TokenData>, Error> {
        let token_data = sqlx::query_as!(
            RawTokenData,
            "SELECT
                t.order_hash, t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.last_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                (
                    t.start_date IS NOT NULL AND t.end_date IS NOT NULL
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN t.start_date AND t.end_date
                    AND t.status != 'CANCELLED'
                ) AS is_listed,
                EXISTS(
                    SELECT 1
                    FROM orderbook_token_offers o
                    WHERE o.token_id = t.token_id
                    AND o.token_address = t.token_address
                    AND o.status not in ('CANCELLED', 'FULFILLED')
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                ) AS has_offer,
                t.currency_chain_id, t.currency_address,
                (SELECT offer_amount FROM orderbook_token_offers WHERE token_id = t.token_id AND token_address = t.token_address AND status = 'PLACED' AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN start_date AND end_date ORDER BY offer_amount DESC LIMIT 1) AS top_bid_amount,
                (SELECT order_hash FROM orderbook_token_offers WHERE token_id = t.token_id AND token_address = t.token_address AND status = 'PLACED' AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN start_date AND end_date ORDER BY offer_amount DESC LIMIT 1) AS top_bid_order_hash,
                t.status,
                t.buy_in_progress
            FROM
                orderbook_token t
            WHERE
                t.token_address = $1;",
            token_address
        ).fetch_all(self).await?;

        let tokens: Vec<TokenData> = token_data.into_iter().map(TokenData::from).collect();

        Ok(tokens)
    }

    async fn get_token_history_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenWithHistory, Error> {
        let token_info = sqlx::query!(
            "SELECT token_id, token_address, current_owner, last_price
             FROM orderbook_token
             WHERE token_id = $1 AND token_address = $2",
            token_id,
            token_address
        )
        .fetch_one(self)
        .await?;

        let history = sqlx::query_as!(
            TokenHistory,
            "SELECT event_type, event_timestamp, order_status,
                    previous_owner, new_owner, amount, canceled_reason,
                    start_date, end_date, end_amount
             FROM orderbook_token_history
             WHERE token_id = $1 AND token_address = $2
             ORDER BY event_timestamp DESC",
            token_id,
            token_address
        )
        .fetch_all(self)
        .await?;

        Ok(TokenWithHistory {
            token_id: token_info.token_id,
            token_address: token_info.token_address,
            current_owner: token_info.current_owner,
            last_price: token_info.last_price,
            history,
        })
    }

    async fn get_token_offers_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenWithOffers, Error> {
        let token_info = sqlx::query!(
            "SELECT token_id, token_address, current_owner, last_price
             FROM orderbook_token
             WHERE token_id = $1 AND token_address = $2",
            token_id,
            token_address
        )
        .fetch_one(self)
        .await?;

        let offers = sqlx::query_as!(
            TokenOffer,
            "SELECT order_hash, offer_maker, offer_amount, offer_quantity, offer_timestamp, currency_chain_id, currency_address, start_date, end_date, status
            FROM orderbook_token_offers
            WHERE token_id = $1 AND token_address = $2
            AND status = 'PLACED'
            ORDER BY offer_timestamp DESC;",
            token_id,
            token_address
        )
        .fetch_all(self)
        .await?;

        Ok(TokenWithOffers {
            token_id: token_info.token_id,
            token_address: token_info.token_address,
            current_owner: token_info.current_owner,
            last_price: token_info.last_price,
            offers,
        })
    }

    async fn delete_token_data(&self, token_address: &str, token_id: &str) -> Result<u64, Error> {
        let mut total_rows_affected = 0;
        let order_hashes = sqlx::query!(
            "SELECT order_hash FROM orderbook_token WHERE token_address = $1 AND token_id = $2",
            token_address,
            token_id
        )
        .fetch_all(self)
        .await?
        .iter()
        .map(|record| record.order_hash.clone())
        .collect::<Vec<String>>();

        sqlx::query!(
            "DELETE FROM orderbook_token_offers WHERE token_address = $1 AND token_id = $2",
            token_address,
            token_id
        )
        .execute(self)
        .await?;

        sqlx::query!(
            "DELETE FROM orderbook_token_history WHERE token_address = $1 AND token_id = $2",
            token_address,
            token_id
        )
        .execute(self)
        .await?;

        sqlx::query!(
            "DELETE FROM orderbook_token WHERE token_address = $1 AND token_id = $2",
            token_address,
            token_id
        )
        .execute(self)
        .await?;

        for order_hash in order_hashes {
            let tables = vec![
                "orderbook_order_cancelled",
                "orderbook_order_created",
                "orderbook_order_executed",
                "orderbook_order_fulfilled",
                "orderbook_order_status",
            ];

            for table in tables {
                let rows_affected =
                    sqlx::query(format!("DELETE FROM {} WHERE order_hash = $1", table).as_str())
                        .bind(&order_hash)
                        .execute(self)
                        .await?
                        .rows_affected();
                total_rows_affected += rows_affected;
            }
        }

        Ok(total_rows_affected)
    }
    async fn flush_all_data(&self) -> Result<u64, Error> {
        let mut total_rows_affected = 0;

        let tables = vec![
            "orderbook_token_offers",
            "orderbook_token_history",
            "orderbook_token",
            "orderbook_order_cancelled",
            "orderbook_order_created",
            "orderbook_order_executed",
            "orderbook_order_fulfilled",
            "orderbook_order_status",
        ];

        for table in tables {
            let rows_affected = sqlx::query(format!("DELETE FROM {}", table).as_str())
                .execute(self)
                .await?
                .rows_affected();
            total_rows_affected += rows_affected;
        }

        Ok(total_rows_affected)
    }

    async fn delete_migrations(&self) -> Result<u64, Error> {
        sqlx::query!("DELETE FROM _sqlx_migrations WHERE version > 0;",)
            .execute(self)
            .await?;

        Ok(1)
    }
}

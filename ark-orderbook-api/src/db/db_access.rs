use crate::models::token::{
    TokenData, TokenHistory, TokenOffer, TokenWithHistory, TokenWithOffers,
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
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_token_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenData, Error> {
        let token_data = sqlx::query_as!(
            TokenData,
            "SELECT
                t.order_hash, t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.current_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                (
                    t.start_date IS NOT NULL AND t.end_date IS NOT NULL
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN t.start_date AND t.end_date
                    AND t.status = 'EXECUTED'
                ) AS is_listed,
                EXISTS(
                        SELECT 1
                        FROM orderbook_token_offers o
                        WHERE o.token_id = t.token_id
                        AND o.token_address = t.token_address
                        AND o.status != 'CANCELLED'
                        AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                    ) AS has_offer,
                t.currency_chain_id, t.currency_address,
                (
                    SELECT MAX(offer.offer_amount)
                    FROM orderbook_token_offers AS offer
                    WHERE offer.token_id = t.token_id
                    AND offer.token_address = t.token_address
                    AND offer.status = 'PLACED'
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN offer.start_date AND offer.end_date
                ) AS top_bid,
                t.status,
                EXISTS(
                    SELECT 1
                    FROM orderbook_token_offers o
                    WHERE o.token_id = t.token_id
                    AND o.token_address = t.token_address
                    AND o.status = 'FULFILLED'
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                ) AS buy_in_progress
            FROM
                orderbook_token t
            LEFT JOIN
                orderbook_token_history th ON th.token_id = t.token_id AND th.token_address = t.token_address
            WHERE
                t.token_address = $1 AND t.token_id = $2
            GROUP BY
                t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.current_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,th.event_type, th.event_timestamp
            ORDER BY
                th.event_timestamp DESC;",
            token_address,
            token_id
        ).fetch_one(self).await?;

        Ok(token_data)
    }

    async fn get_tokens_by_owner_data(&self, owner: &str) -> Result<Vec<TokenData>, Error> {
        let tokens_data = sqlx::query_as!(
            TokenData,
            "SELECT
                t.order_hash, t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.current_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                (
                    t.start_date IS NOT NULL AND t.end_date IS NOT NULL
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN t.start_date AND t.end_date
                    AND t.status = 'EXECUTED'
                ) AS is_listed,
                EXISTS(
                        SELECT 1
                        FROM orderbook_token_offers o
                        WHERE o.token_id = t.token_id
                        AND o.token_address = t.token_address
                        AND o.status = 'EXECUTED'
                        AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                    ) AS has_offer,
                t.currency_chain_id, t.currency_address,
                (
                    SELECT MAX(offer.offer_amount)
                    FROM orderbook_token_offers AS offer
                    WHERE offer.token_id = t.token_id
                    AND offer.token_address = t.token_address
                    AND offer.status = 'PLACED'
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN offer.start_date AND offer.end_date
                ) AS top_bid,
                t.status,
                EXISTS(
                    SELECT 1
                    FROM orderbook_token_offers o
                    WHERE o.token_id = t.token_id
                    AND o.token_address = t.token_address
                    AND o.status = 'FULFILLED'
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                ) AS buy_in_progress
            FROM
                orderbook_token t
            LEFT JOIN
                orderbook_token_history th ON th.token_id = t.token_id AND th.token_address = t.token_address
            WHERE
                t.current_owner = $1
            GROUP BY
                t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.current_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,th.event_type, th.event_timestamp
            ORDER BY
                th.event_timestamp DESC;",
            owner
        ).fetch_all(self).await?;

        Ok(tokens_data)
    }

    async fn get_token_by_collection_data(
        &self,
        token_address: &str,
    ) -> Result<Vec<TokenData>, Error> {
        let token_data = sqlx::query_as!(
            TokenData,
            "SELECT
                t.order_hash, t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.current_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                (
                    t.start_date IS NOT NULL AND t.end_date IS NOT NULL
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN t.start_date AND t.end_date
                    AND t.status = 'EXECUTED'
                ) AS is_listed,
                EXISTS(
                    SELECT 1
                    FROM orderbook_token_offers o
                    WHERE o.token_id = t.token_id
                    AND o.token_address = t.token_address
                    AND o.status = 'EXECUTED'
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                ) AS has_offer,
                t.currency_chain_id, t.currency_address,
                (
                    SELECT MAX(offer.offer_amount)
                    FROM orderbook_token_offers AS offer
                    WHERE offer.token_id = t.token_id
                    AND offer.token_address = t.token_address
                    AND offer.status = 'PLACED'
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN offer.start_date AND offer.end_date
                ) AS top_bid,
                t.status,
                EXISTS(
                    SELECT 1
                    FROM orderbook_token_offers o
                    WHERE o.token_id = t.token_id
                    AND o.token_address = t.token_address
                    AND o.status = 'FULFILLED'
                    AND EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN o.start_date AND o.end_date
                ) AS buy_in_progress
            FROM
                orderbook_token t
            WHERE
                t.token_address = $1;",
            token_address
        ).fetch_all(self).await?;

        Ok(token_data)
    }

    async fn get_token_history_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenWithHistory, Error> {
        let token_info = sqlx::query!(
            "SELECT token_id, token_address, current_owner, current_price
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
                    previous_owner, new_owner, amount, canceled_reason
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
            current_price: token_info.current_price,
            history,
        })
    }

    async fn get_token_offers_data(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<TokenWithOffers, Error> {
        let token_info = sqlx::query!(
            "SELECT token_id, token_address, current_owner, current_price
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
            current_price: token_info.current_price,
            offers,
        })
    }
}

#[cfg(test)]
pub struct MockDb;

#[cfg(test)]
#[async_trait]
impl DatabaseAccess for MockDb {
    async fn get_token_data(
        &self,
        _token_address: &str,
        _token_id: &str,
    ) -> Result<TokenData, Error> {
        Ok(TokenData {
            order_hash: "0x12345".to_string(),
            token_chain_id: "chainXYZ".to_string(),
            token_address: "0xABCDEF123456".to_string(),
            token_id: "token789".to_string(),
            listed_timestamp: 1234567890,
            updated_timestamp: 1234567891,
            current_owner: "owner123".to_string(),
            current_price: Some("100".to_string()),
            quantity: Some("10".to_string()),
            start_amount: Some("50".to_string()),
            end_amount: Some("150".to_string()),
            start_date: Some(1234567890),
            end_date: Some(1234567891),
            is_listed: Some(true),
            has_offer: Some(false),
            broker_id: Some("brokerXYZ".to_string()),
            currency_address: Some("0xABCDEF123456".to_string()),
            currency_chain_id: Some("chainXYZ".to_string()),
            top_bid: Some("100".to_string()),
            status: "EXECUTED".to_string(),
            buy_in_progress: Some(false),
        })
    }

    async fn get_token_by_collection_data(
        &self,
        _token_address: &str,
    ) -> Result<Vec<TokenData>, Error> {
        Ok(vec![
            TokenData {
                order_hash: "0x123".to_string(),
                token_chain_id: "chainXYZ".to_string(),
                token_address: "0xABCDEF123456".to_string(),
                token_id: "token789".to_string(),
                listed_timestamp: 1234567890,
                updated_timestamp: 1234567891,
                current_owner: "owner123".to_string(),
                current_price: Some("100".to_string()),
                quantity: Some("10".to_string()),
                start_amount: Some("50".to_string()),
                end_amount: Some("150".to_string()),
                start_date: Some(1234567890),
                end_date: Some(1234567891),
                is_listed: Some(true),
                has_offer: Some(false),
                broker_id: Some("brokerXYZ".to_string()),
                currency_address: Some("0xABCDEF123456".to_string()),
                currency_chain_id: Some("chainXYZ".to_string()),
                top_bid: Some("100".to_string()),
                status: "PLACED".to_string(),
                buy_in_progress: Some(false),
            },
            TokenData {
                order_hash: "0x1234".to_string(),
                token_chain_id: "chainWXYZ".to_string(),
                token_address: "0xABCDEF1234567".to_string(),
                token_id: "token7890".to_string(),
                listed_timestamp: 1234567890,
                updated_timestamp: 1234567891,
                current_owner: "owner1234".to_string(),
                current_price: Some("100".to_string()),
                quantity: Some("10".to_string()),
                start_amount: Some("50".to_string()),
                end_amount: Some("150".to_string()),
                start_date: Some(1234567890),
                end_date: Some(1234567891),
                is_listed: Some(true),
                has_offer: Some(false),
                broker_id: Some("brokerXYZ".to_string()),
                currency_address: Some("0xABCDEF123456".to_string()),
                currency_chain_id: Some("chainXYZ".to_string()),
                top_bid: Some("100".to_string()),
                status: "PLACED".to_string(),
                buy_in_progress: Some(false),
            },
        ])
    }

    async fn get_token_history_data(
        &self,
        _token_address: &str,
        _token_id: &str,
    ) -> Result<TokenWithHistory, Error> {
        let history = vec![TokenHistory {
            event_type: "Listing".to_string(),
            event_timestamp: 1234567890,
            order_status: "Active".to_string(),
            previous_owner: None,
            new_owner: Some("owner123".to_string()),
            amount: Some("100".to_string()),
            canceled_reason: None,
        }];

        Ok(TokenWithHistory {
            token_address: "0xABCDEF123456".to_string(),
            token_id: "token789".to_string(),
            current_owner: "owner123".to_string(),
            current_price: Some("100".to_string()),
            history,
        })
    }

    async fn get_token_offers_data(
        &self,
        _token_address: &str,
        _token_id: &str,
    ) -> Result<TokenWithOffers, Error> {
        let offers = vec![TokenOffer {
            order_hash: "0x123".to_string(),
            offer_maker: "maker123".to_string(),
            offer_amount: "100".to_string(),
            offer_quantity: "10".to_string(),
            offer_timestamp: 1234567890,
            start_date: 1234567890,
            end_date: 1234567899,
            currency_address: Some("0xABCDEF123456".to_string()),
            currency_chain_id: Some("chainXYZ".to_string()),
            status: "EXECUTED".to_string(),
        }];
        Ok(TokenWithOffers {
            token_address: "0xABCDEF123456".to_string(),
            token_id: "token789".to_string(),
            current_owner: "owner123".to_string(),
            current_price: Some("100".to_string()),
            offers,
        })
    }

    async fn get_tokens_by_owner_data(&self, _owner: &str) -> Result<Vec<TokenData>, Error> {
        Ok(vec![
            TokenData {
                order_hash: "0x123".to_string(),
                token_chain_id: "chainXYZ".to_string(),
                token_address: "0xABCDEF123456".to_string(),
                token_id: "token789".to_string(),
                listed_timestamp: 1234567890,
                updated_timestamp: 1234567891,
                current_owner: "owner123".to_string(),
                current_price: Some("100".to_string()),
                quantity: Some("10".to_string()),
                start_amount: Some("50".to_string()),
                end_amount: Some("150".to_string()),
                start_date: Some(1234567890),
                end_date: Some(1234567891),
                broker_id: Some("brokerXYZ".to_string()),
                is_listed: None,
                has_offer: None,
                currency_chain_id: Some("chainXYZ".to_string()),
                currency_address: Some("0xABCDEF123456".to_string()),
                top_bid: Some("100".to_string()),
                status: "EXECUTED".to_string(),
                buy_in_progress: Some(false),
            },
            TokenData {
                order_hash: "0x123".to_string(),
                token_chain_id: "chainWXYZ".to_string(),
                token_address: "0xABCDEF1234567".to_string(),
                token_id: "token7890".to_string(),
                listed_timestamp: 2234567890,
                updated_timestamp: 2234567891,
                current_owner: "owner1234".to_string(),
                current_price: Some("200".to_string()),
                quantity: Some("20".to_string()),
                start_amount: Some("100".to_string()),
                end_amount: Some("300".to_string()),
                start_date: Some(2234567890),
                end_date: Some(2234567891),
                broker_id: Some("brokerWXYZ".to_string()),
                is_listed: None,
                has_offer: None,
                currency_chain_id: Some("chainWXYZ".to_string()),
                currency_address: Some("0xABCDEF1234567".to_string()),
                top_bid: Some("50".to_string()),
                status: "EXECUTED".to_string(),
                buy_in_progress: Some(true),
            },
        ])
    }
}

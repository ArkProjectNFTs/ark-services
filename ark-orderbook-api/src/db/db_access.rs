use async_trait::async_trait;
use crate::models::token::TokenData;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
pub trait DatabaseAccess: Send + Sync {
    async fn get_token_data(&self, token_address: &str, token_id: &str) -> Result<TokenData, Error>;
    async fn get_token_by_collection_data(&self, token_address: &str) -> Result<Vec<TokenData>, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_token_data(&self,
                            token_address: &str,
                            token_id: &str) -> Result<TokenData, Error> {

        let token_data = sqlx::query_as!(
            TokenData,
            "SELECT
                t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.current_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                EXISTS(SELECT 1 FROM orderbook_token_history th
                           WHERE th.token_id = t.token_id AND th.token_address = t.token_address
                           AND th.event_type = 'Listing'
                           AND th.event_timestamp < COALESCE((SELECT MAX(th2.event_timestamp) FROM orderbook_token_history th2
                                                             WHERE th2.token_id = t.token_id AND th2.token_address = t.token_address
                                                             AND th2.event_type = 'Offer'), th.event_timestamp)
                          ) AS is_listed,
                CASE WHEN COALESCE(MAX(CASE WHEN th.event_type = 'Offer' AND th.event_timestamp > (SELECT MAX(th2.event_timestamp) FROM orderbook_token_history th2 WHERE th2.event_type = 'Listing' AND th2.token_id = t.token_id AND th2.token_address = t.token_address) THEN 1 ELSE 0 END) OVER (PARTITION BY t.token_id, t.token_address ORDER BY th.event_timestamp DESC), 0) = 1 THEN TRUE ELSE FALSE END AS has_offer
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

    async fn get_token_by_collection_data(&self,
                                          token_address: &str) -> Result<Vec<TokenData>, Error> {
        let token_data = sqlx::query_as!(
            TokenData,
            "SELECT
                t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                t.updated_timestamp, t.current_owner, t.current_price,
                t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                t.broker_id,
                COALESCE(
                    (SELECT MAX(th.event_timestamp)
                     FROM orderbook_token_history th
                     WHERE th.token_id = t.token_id AND th.token_address = t.token_address AND th.event_type = 'Listing'
                    ) <
                    (SELECT MAX(th.event_timestamp)
                     FROM orderbook_token_history th
                     WHERE th.token_id = t.token_id AND th.token_address = t.token_address AND th.event_type = 'Offer'
                    ), FALSE) AS is_listed,
                EXISTS(
                    SELECT 1
                    FROM orderbook_token_history th
                    WHERE th.token_id = t.token_id AND th.token_address = t.token_address AND th.event_type = 'Offer'
                ) AS has_offer
            FROM
                orderbook_token t
            WHERE
                t.token_address = $1;",
            token_address
        ).fetch_all(self).await?;

        Ok(token_data)
    }
}

#[cfg(test)]
pub struct MockDb;

#[cfg(test)]
#[async_trait]
impl DatabaseAccess for MockDb {
    async fn get_token_data(&self, _token_address: &str, _token_id: &str) -> Result<TokenData, Error> {
        Ok(TokenData {
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
        })
    }

    async fn get_token_by_collection_data(&self, _token_address: &str) -> Result<Vec<TokenData>, Error> {
        Ok(vec![TokenData {
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
        },
        TokenData {
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
        }])
    }
}

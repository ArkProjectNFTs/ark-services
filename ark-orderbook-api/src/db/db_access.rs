use async_trait::async_trait;
use crate::models::token::TokenData;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
pub trait DatabaseAccess: Send + Sync {
    async fn get_token_data(&self, token_address: &str, token_id: &str) -> Result<TokenData, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_token_data(&self,
                            token_address: &str,
                            token_id: &str) -> Result<TokenData, Error> {

        let token_data = sqlx::query_as!(
            TokenData,
            "SELECT t.token_chain_id, t.token_id, t.token_address, t.listed_timestamp,
                    t.updated_timestamp, t.current_owner, t.current_price,
                    t.quantity, t.start_amount, t.end_amount, t.start_date, t.end_date,
                    t.broker_id,
                    CASE
                        WHEN th.event_type = 'Listing' THEN TRUE
                        ELSE FALSE
                    END AS is_listed
             FROM orderbook_token t
             LEFT JOIN LATERAL (
                 SELECT th.event_type
                 FROM orderbook_token_history th
                 WHERE th.token_id = t.token_id AND th.token_address = t.token_address
                 ORDER BY th.event_timestamp DESC
                 LIMIT 1
             ) th ON TRUE
             WHERE t.token_address = $1 AND t.token_id = $2",
            token_address,
            token_id
        ).fetch_one(self).await?;

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
            broker_id: Some("brokerXYZ".to_string()),
        })
    }
}

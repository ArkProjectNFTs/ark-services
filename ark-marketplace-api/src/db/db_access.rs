use crate::models::collection::CollectionData;
use async_trait::async_trait;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
pub trait DatabaseAccess: Send + Sync {
    async fn get_collection_data(&self, collection_address: &str) -> Result<CollectionData, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_collection_data(&self, contract_address: &str) -> Result<CollectionData, Error> {
        let collection_data = sqlx::query_as!(
            CollectionData,
            "SELECT
                 contract_image AS image,
                 contract_name AS collection_name,
                 (
                     SELECT MIN(listing_start_amount)
                     FROM token
                     WHERE token.contract_address = contract.contract_address
                     AND token.chain_id = contract.chain_id
                     AND token.listing_timestamp <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) AS floor,
                 CAST(0 AS INTEGER) AS floor_7d_percentage,
                 CAST(0 AS INTEGER) AS volume_7d_eth,
                 (
                     SELECT MAX(offer_amount)
                     FROM token_offer
                     WHERE token_offer.contract_address = contract.contract_address
                     AND token_offer.chain_id = contract.chain_id
                 ) AS top_offer,
                 (
                     SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sell'
                     AND token_event.block_timestamp >= (EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')::BIGINT)
                 ) AS sales_7d,
                 CAST(0 AS INTEGER) AS marketcap,
                 (
                     SELECT COUNT(*)
                     FROM token
                     WHERE token.contract_address = contract.contract_address
                     AND token.chain_id = contract.chain_id
                     AND token.listing_timestamp <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) AS listed_items,
                (
                    SELECT COUNT(*)
                    FROM token
                    WHERE token.contract_address = contract.contract_address
                    AND token.chain_id = contract.chain_id
                    AND token.listing_timestamp <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                    AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                ) * 100 / NULLIF(
                    (
                        SELECT COUNT(*)
                        FROM token
                        WHERE token.contract_address = contract.contract_address
                        AND token.chain_id = contract.chain_id
                    ), 0
                ) AS listed_percentage
                FROM
                 contract
             WHERE
                 contract_address = $1",
            contract_address,
        ).fetch_one(self).await?;

        Ok(collection_data)
    }
}

#[cfg(test)]
pub struct MockDb;

#[cfg(test)]
#[async_trait]
impl DatabaseAccess for MockDb {
    async fn get_collection_data(&self, _contract_address: &str) -> Result<CollectionData, Error> {
        Ok(CollectionData {
            image: "https://example.com/image.png".to_string(),
            collection_name: "Example Collection".to_string(),
            floor: 1.23,
            floor_7d_percentage: 4.56,
            volume_7d_eth: 789,
            top_offer: Some("Top Offer".to_string()),
            sales_7d: 10,
            marketcap: 1112,
            listed_items: 13,
            listed_percentage: 14,
        })
    }
}

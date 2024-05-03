use crate::models::collection::CollectionData;
use async_trait::async_trait;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
pub trait DatabaseAccess: Send + Sync {
    async fn get_collection_data(
        &self,
        page: i64,
        items_per_page: i64,
    ) -> Result<Vec<CollectionData>, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_collection_data(
        &self,
        page: i64,
        items_per_page: i64,
    ) -> Result<Vec<CollectionData>, Error> {
        let collection_data = sqlx::query_as!(
            CollectionData,
            "SELECT
                 contract_image AS image,
                 contract_name AS collection_name,
                 (
                     SELECT MIN(listing_start_amount)
                     FROM token
                     WHERE token.contract_id = contract.contract_id
                     AND token.listing_timestamp <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) AS floor,
                 CAST(0 AS INTEGER) AS floor_7d_percentage,
                 CAST(0 AS INTEGER) AS volume_7d_eth,
                 (
                     SELECT MAX(offer_amount)
                     FROM token_offers
                     WHERE token_offers.contract_id = contract.contract_id
                 ) AS top_offer,
                 (
                     SELECT COUNT(*)
                     FROM token_events
                     WHERE token_events.contract_id = contract.contract_id
                     AND token_events.event_type = 'Sell'
                     AND token_events.timestamp >= (EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')::BIGINT)
                 ) AS sales_7d,
                 CAST(0 AS INTEGER) AS marketcap,
                 (
                     SELECT COUNT(*)
                     FROM token
                     WHERE token.contract_id = contract.contract_id
                     AND token.listing_timestamp <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) AS listed_items,
                (
                    SELECT COUNT(*)
                    FROM token
                    WHERE token.contract_id = contract.contract_id
                    AND token.listing_timestamp <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                    AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                ) * 100 / NULLIF(
                    (
                        SELECT COUNT(*)
                        FROM token
                        WHERE token.contract_id = contract.contract_id
                    ), 0
                ) AS listed_percentage
                FROM
                 contract
           LIMIT $1 OFFSET $2",
           items_per_page,
           (page - 1) * items_per_page
        ).fetch_all(self).await?;
        // @TODO : should we filter by symbol ETH or STRK ?

        Ok(collection_data)
    }
}

#[cfg(test)]
pub struct MockDb;

#[cfg(test)]
#[async_trait]
impl DatabaseAccess for MockDb {
    async fn get_collection_data(
        &self,
        page: i64,
        items_per_page: i64,
    ) -> Result<Vec<CollectionData>, Error> {
        Ok(vec![CollectionData {
            image: Some("https://example.com/image.png".to_string()),
            collection_name: Some("Example Collection".to_string()),
            floor: Some("1".to_string()),
            floor_7d_percentage: Some(4),
            volume_7d_eth: Some(789),
            top_offer: Some("Top Offer".to_string()),
            sales_7d: Some(10),
            marketcap: Some(1112),
            listed_items: Some(13),
            listed_percentage: Some(14),
        }])
    }
}

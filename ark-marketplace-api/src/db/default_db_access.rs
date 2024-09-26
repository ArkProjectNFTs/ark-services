use crate::db::db_access::LISTING_TYPE_AUCTION_STR;
use crate::models::default::{LastSale, LiveAuction};
use async_trait::async_trait;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait DatabaseAccess: Send + Sync {
    async fn get_last_sales(&self) -> Result<Vec<LastSale>, Error>;
    async fn get_live_auctions(&self) -> Result<Vec<LiveAuction>, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_last_sales(&self) -> Result<Vec<LastSale>, Error> {
        let recent_sales_query = r#"
            SELECT
                t.metadata,
                c.contract_name AS collection_name,
                t.contract_address AS collection_address,
                te.amount AS price,
                te.from_address AS from,
                te.to_address AS to,
                te.block_timestamp AS timestamp,
                te.transaction_hash
            FROM
                token_event te
            LEFT JOIN
                token t ON te.contract_address = t.contract_address
                    AND te.chain_id = t.chain_id
                    AND te.token_id = t.token_id
            LEFT JOIN contract c ON te.contract_address = c.contract_address
                    AND te.chain_id = c.chain_id
            WHERE
                te.event_type = 'Sale'
            ORDER BY
                te.block_timestamp DESC
            LIMIT 12
        "#;

        // Execute the query
        let last_sales = sqlx::query_as::<_, LastSale>(&recent_sales_query)
            .fetch_all(self)
            .await?;

        Ok(last_sales)
    }

    async fn get_live_auctions(&self) -> Result<Vec<LiveAuction>, Error> {
        let live_auctions_query_template = r#"
            SELECT
                t.metadata,
                t.listing_end_date as end_timestamp
            FROM
                token t
            WHERE
                t.listing_start_date IS NOT NULL
              AND t.listing_type = '{}'
            ORDER BY
                t.listing_end_date DESC
            LIMIT 6
        "#;

        let live_auctions_query = format!(
            "{}",
            live_auctions_query_template.replace("{}", LISTING_TYPE_AUCTION_STR)
        );
        // Execute the query
        let live_auctions = sqlx::query_as::<_, LiveAuction>(&live_auctions_query)
            .fetch_all(self)
            .await?;

        Ok(live_auctions)
    }
}

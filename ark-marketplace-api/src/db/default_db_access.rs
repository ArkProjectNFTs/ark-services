use crate::models::default::LastSale;
use async_trait::async_trait;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait DatabaseAccess: Send + Sync {
    async fn get_last_sales(&self) -> Result<Vec<LastSale>, Error>;
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
}

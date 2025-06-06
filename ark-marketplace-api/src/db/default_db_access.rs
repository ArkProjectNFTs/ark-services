use crate::db::db_access::LISTING_TYPE_AUCTION_STR;
use crate::handlers::utils::CHAIN_ID;
use crate::models::default::{
    CollectionInfo, Currency, LastSale, LastSaleDB, LiveAuction, PreviewNft, Trending,
};
use async_trait::async_trait;
use sqlx::Error;
use sqlx::PgPool;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait DatabaseAccess: Send + Sync {
    async fn get_last_sales(&self) -> Result<Vec<LastSale>, Error>;
    async fn get_live_auctions(&self) -> Result<Vec<LiveAuction>, Error>;

    async fn get_trending(&self, time_range: &str) -> Result<Vec<Trending>, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_last_sales(&self) -> Result<Vec<LastSale>, Error> {
        let currencies: Vec<Currency> = sqlx::query_as!(
            Currency,
            r#"
            SELECT currency_address as contract, symbol, decimals
            FROM public.currency_mapping
            "#
        )
        .fetch_all(self)
        .await?;

        let recent_sales_query = r#"
            SELECT
                t.metadata,
                c.contract_name AS collection_name,
                t.contract_address AS collection_address,
                hex_to_decimal(te.amount) AS price,
                te.from_address AS from,
                te.to_address AS to,
                te.block_timestamp AS timestamp,
                te.transaction_hash,
                te.token_id,
                te.currency_address
            FROM
                token_event te
            LEFT JOIN
                token t ON te.contract_address = t.contract_address
                    AND te.chain_id = t.chain_id
                    AND te.token_id = t.token_id
            LEFT JOIN contract c ON te.contract_address = c.contract_address
                    AND te.chain_id = c.chain_id
            WHERE
                te.event_type IN ('Sale', 'Executed')
            ORDER BY
                te.block_timestamp DESC
            LIMIT 12
        "#;

        // Execute the query
        let last_sales_db = sqlx::query_as::<_, LastSaleDB>(recent_sales_query)
            .fetch_all(self)
            .await?;

        let last_sales: Vec<LastSale> = last_sales_db
            .into_iter()
            .map(|sale| {
                let currency = currencies
                    .iter()
                    .find(|c| c.contract == sale.currency_address)
                    .cloned();
                LastSale {
                    metadata: sale.metadata,
                    collection_name: sale.collection_name,
                    collection_address: sale.collection_address,
                    price: sale.price,
                    from: sale.from,
                    to: sale.to,
                    timestamp: sale.timestamp,
                    transaction_hash: sale.transaction_hash,
                    token_id: sale.token_id,
                    currency,
                }
            })
            .collect();

        Ok(last_sales)
    }

    async fn get_live_auctions(&self) -> Result<Vec<LiveAuction>, Error> {
        let live_auctions_query_template = r#"
            SELECT
                t.metadata,
                t.listing_end_date as end_timestamp,
                t.contract_address as collection_address,
                t.token_id
            FROM
                token t
            WHERE
                t.listing_start_date IS NOT NULL
              AND t.listing_type = '{}'
            ORDER BY
                t.listing_end_date DESC
            LIMIT 6
        "#;

        let live_auctions_query = live_auctions_query_template
            .replace("{}", LISTING_TYPE_AUCTION_STR)
            .to_string();
        // Execute the query
        let live_auctions = sqlx::query_as::<_, LiveAuction>(&live_auctions_query)
            .fetch_all(self)
            .await?;

        Ok(live_auctions)
    }

    async fn get_trending(&self, time_range: &str) -> Result<Vec<Trending>, Error> {
        let contract_timestamp_clause: String = if time_range.is_empty() {
            String::new()
        } else {
            format!(" AND contract_marketdata.timerange = '{}'", time_range)
        };

        let sql_query = format!(
            "SELECT
                    contract.contract_address as collection_address,
                    contract_image as collection_image,
                    contract_name as collection_name,
                    floor_price,
                    contract_marketdata.floor_percentage as floor_difference
                    FROM
                     contract
                     INNER JOIN contract_marketdata on contract.contract_address = contract_marketdata.contract_address and contract.chain_id = contract_marketdata.chain_id {}
                     WHERE contract_marketdata.volume > 0
               GROUP BY contract.contract_address, contract.chain_id, floor_difference, volume
               ORDER BY VOLUME DESC
               LIMIT 5
               ",
            contract_timestamp_clause,
        );

        let mut collection_data: Vec<CollectionInfo> =
            sqlx::query_as(&sql_query).fetch_all(self).await?;

        // Check if we have less than 5 results and fill up if necessary
        if collection_data.len() < 5 {
            let missing_count = 5 - collection_data.len();

            // Get the contract addresses already included in collection_data
            let existing_addresses: Vec<&str> = collection_data
                .iter()
                .map(|trending| trending.collection_address.as_str())
                .collect();

            // Create SQL query to fetch additional collections
            let additional_sql_query = format!(
                "SELECT
                    contract.contract_address as collection_address,
                    contract_image as collection_image,
                    contract_name as collection_name,
                    floor_price,
                    volume,
                    contract_marketdata.floor_percentage as floor_difference
                FROM
                    contract
                    INNER JOIN contract_marketdata ON contract.contract_address = contract_marketdata.contract_address
                    AND contract.chain_id = contract_marketdata.chain_id
                WHERE
                    contract_marketdata.volume > 0
                    AND contract.contract_address NOT IN ({})
                GROUP BY
                    contract.contract_address, contract.chain_id, floor_difference, volume
                ORDER BY volume DESC
                LIMIT {}",
                existing_addresses.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                missing_count
            );
            // Execute the query to fetch additional collections
            let additional_collections: Vec<CollectionInfo> = sqlx::query_as(&additional_sql_query)
                .bind(existing_addresses)
                .fetch_all(self)
                .await?;
            // Append additional collections to the collection_data
            collection_data.extend(additional_collections);
        }

        let mut trending_data: Vec<Trending> = Vec::with_capacity(5);

        // Retrieve preview NFTs for each collection
        for collection in collection_data {
            let preview_nft_sql = "SELECT
                    metadata
                 FROM
                    token
                 WHERE
                    contract_address = $1
                 AND chain_id = $2
                 AND metadata is not null
                 LIMIT 3";

            let preview_nfts: Vec<PreviewNft> = sqlx::query_as(preview_nft_sql)
                .bind(&collection.collection_address)
                .bind(CHAIN_ID)
                .fetch_all(self)
                .await?;

            trending_data.push(Trending {
                collection_address: collection.collection_address.clone(),
                collection_image: collection.collection_image.clone(),
                collection_name: collection.collection_name.clone(),
                floor_price: collection.floor_price.clone(),
                floor_difference: collection.floor_difference,
                preview_nfts,
            });
        }

        Ok(trending_data)
    }
}

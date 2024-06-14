use crate::models::collection::{CollectionData, CollectionPortfolioData};
use crate::models::token::{TokenData, TokenPortfolioData};
use async_trait::async_trait;
use sqlx::Error;
use sqlx::PgPool;
use sqlx::Row;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait DatabaseAccess: Send + Sync {
    async fn get_tokens_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        page: i64,
        items_per_page: i64,
        buy_now: bool,
        sort: &str,
        direction: &str,
    ) -> Result<(Vec<TokenData>, bool, i64), Error>;

    async fn get_tokens_portfolio_data(
        &self,
        user_address: &str,
        page: i64,
        items_per_page: i64,
        buy_now: bool,
        sort: &str,
        direction: &str,
        collection: &str,
    ) -> Result<(Vec<TokenPortfolioData>, bool, i64), Error>;

    async fn get_collections_data(
        &self,
        page: i64,
        items_per_page: i64,
        time_range: &str,
        user_address: Option<&str>,
    ) -> Result<Vec<CollectionData>, Error>;

    async fn get_portfolio_collections_data(
        &self,
        page: i64,
        items_per_page: i64,
        user_address: &str,
    ) -> Result<(Vec<CollectionPortfolioData>, bool, i64), Error>;

    async fn get_collection_data(
        &self,
        contract_address: &str,
        chain_id: &str,
    ) -> Result<CollectionData, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_collections_data(
        &self,
        page: i64,
        items_per_page: i64,
        time_range: &str,
        user_address: Option<&str>,
    ) -> Result<Vec<CollectionData>, Error> {
        let user_clause = match user_address {
            Some(address) => format!(" AND token.current_owner = '{}'", address),
            None => String::new(),
        };

        let interval = match time_range {
            "10m" => "INTERVAL '10 minutes'",
            "1h" => "INTERVAL '1 hour'",
            "6h" => "INTERVAL '6 hours'",
            "1D" => "INTERVAL '1 day'",
            "7D" => "INTERVAL '7 days'",
            "30D" => "INTERVAL '30 days'",
            _ => "",
        };

        let contract_timestamp_clause: String = if interval.is_empty() {
            String::new()
        } else {
            format!(
                " AND contract.updated_timestamp >= (EXTRACT(EPOCH FROM NOW() - {})::BIGINT)",
                interval
            )
        };

        let sql_query = format!(
                "SELECT
                     contract.contract_address as address,
                     contract_image AS image,
                     contract_name AS collection_name,
                     (
                         SELECT MIN(listing_start_amount)
                         FROM token
                         WHERE token.contract_address = contract.contract_address
                         AND token.chain_id = contract.chain_id
                         AND token.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                         AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                     ) AS floor,
                     CAST(0 AS INTEGER) AS floor_7d_percentage,
                     CAST(0 AS INTEGER) AS volume_7d_eth,
                     (
                         SELECT COALESCE(MAX(CAST(offer_amount AS BIGINT)), 0)
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
                         AND token.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                         AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                     ) AS listed_items,
                    (
                        SELECT COUNT(*)
                        FROM token
                        WHERE token.contract_address = contract.contract_address
                        AND token.chain_id = contract.chain_id
                        AND token.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                        AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                    ) * 100 / NULLIF(
                        (
                            SELECT COUNT(*)
                            FROM token
                            WHERE token.contract_address = contract.contract_address
                            AND token.chain_id = contract.chain_id
                        ), 0
                    ) AS listed_percentage,
                    (
                          SELECT COUNT(*)
                          FROM token
                          WHERE token.contract_address = contract.contract_address
                          AND token.chain_id = contract.chain_id
                      ) AS token_count,
                    (
                       SELECT COUNT(DISTINCT current_owner)
                       FROM token
                       WHERE token.contract_address = contract.contract_address
                       AND token.chain_id = contract.chain_id
                    ) AS owner_count,
                    (
                         SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0)
                         FROM token_event
                         WHERE token_event.contract_address = contract.contract_address
                         AND token_event.chain_id = contract.chain_id
                         AND token_event.event_type = 'Sell'
                    ) AS total_volume,
                    (
                         SELECT COUNT(*)
                         FROM token_event
                         WHERE token_event.contract_address = contract.contract_address
                         AND token_event.chain_id = contract.chain_id
                         AND token_event.event_type = 'Sell'
                     ) AS total_sales,
                    contract.contract_symbol
                    FROM
                     contract
                     INNER JOIN token ON contract.contract_address = token.contract_address
                     WHERE 1=1
                     {} {}
               GROUP BY contract.contract_address, contract.chain_id
               LIMIT {} OFFSET {}
               ",
               contract_timestamp_clause,
               user_clause,
               items_per_page,
               (page - 1) * items_per_page,
            );
        let collection_data = sqlx::query_as::<sqlx::Postgres, CollectionData>(&sql_query)
            .fetch_all(self)
            .await
            .unwrap_or_else(|err| {
                eprintln!("Query error : {}", err);
                std::process::exit(1);
            });

        Ok(collection_data)
    }

    async fn get_portfolio_collections_data(
        &self,
        page: i64,
        items_per_page: i64,
        user_address: &str,
    ) -> Result<(Vec<CollectionPortfolioData>, bool, i64), Error> {
        let total_count = sqlx::query!(
            "
                SELECT COUNT(DISTINCT contract.contract_address)
                FROM contract
                INNER JOIN token ON contract.contract_address = token.contract_address
                WHERE token.current_owner = $1 and contract.is_verified = true
                ",
            user_address
        )
        .fetch_one(self)
        .await?;

        let count = total_count.count.unwrap_or(0);

        let collection_portfolio_data: Vec<CollectionPortfolioData> = sqlx::query_as!(
            CollectionPortfolioData,
            "
            SELECT
                 contract.contract_address as address,
                 contract_image AS image,
                 contract_name AS collection_name,
                 ( SELECT count(*)
                    FROM   token t1
                    WHERE  t1.contract_address = contract.contract_address
                      AND  t1.chain_id = contract.chain_id
                      AND  t1.current_owner = token.current_owner
                 ) as user_token_count,
                 ( SELECT count(*)
                     FROM   token t1
                     WHERE  t1.contract_address = contract.contract_address
                       AND  t1.chain_id = contract.chain_id
                       AND  t1.current_owner = token.current_owner
                       AND t1.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                       AND (t1.listing_end_date IS NULL OR t1.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                  ) as user_listed_tokens,
                 (
                     SELECT COALESCE(MIN(CAST(listing_start_amount AS INTEGER)), 0)
                     FROM token
                     WHERE token.contract_address = contract.contract_address
                     AND token.chain_id = contract.chain_id
                     AND token.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) AS floor,
                 (
                   SELECT COUNT(*)
                   FROM token
                   WHERE token.contract_address = contract.contract_address
                   AND token.chain_id = contract.chain_id
                 ) AS token_count
                FROM
                 contract
                 INNER JOIN token ON contract.contract_address = token.contract_address
                 WHERE token.current_owner = $1
                 AND   contract.is_verified = true
           GROUP BY contract.contract_address, contract.chain_id, token.current_owner
           LIMIT $2 OFFSET $3
           ",
           user_address,
           items_per_page,
           (page - 1) * items_per_page,
        )
        .fetch_all(self)
        .await?;

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;
        Ok((collection_portfolio_data, has_next_page, count))
    }

    async fn get_collection_data(
        &self,
        contract_address: &str,
        chain_id: &str,
    ) -> Result<CollectionData, Error> {
        let collection_data = sqlx::query_as!(
             CollectionData,
             r#"
             SELECT
                 contract.contract_address as address,
                 CASE
                     WHEN contract_image = '' THEN NULL
                     ELSE contract_image
                 END AS image,
                 contract_name AS collection_name,
                 (
                     SELECT COALESCE(MIN(CAST(listing_start_amount AS INTEGER)), 0)
                     FROM token
                     WHERE token.contract_address = $1
                     AND token.chain_id = contract.chain_id
                     AND token.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) AS floor,
                 CAST(0 AS INTEGER) AS floor_7d_percentage,
                 CAST(0 AS INTEGER) AS volume_7d_eth,
                 (
                     SELECT COALESCE(MAX(CAST(offer_amount AS BIGINT)), 0)
                     FROM token_offer
                     WHERE token_offer.contract_address = $1
                     AND token_offer.chain_id = contract.chain_id
                 ) AS top_offer,
                 (
                     SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = $1
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sell'
                     AND token_event.block_timestamp >= (EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')::BIGINT)
                 ) AS sales_7d,
                 CAST(0 AS INTEGER) AS marketcap,
                 (
                     SELECT COUNT(*)
                     FROM token
                     WHERE token.contract_address = $1
                     AND token.chain_id = contract.chain_id
                     AND token.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) AS listed_items,
                 (
                     SELECT COUNT(*)
                     FROM token
                     WHERE token.contract_address = $1
                     AND token.chain_id = contract.chain_id
                     AND token.listing_start_date <= (EXTRACT(EPOCH FROM NOW())::BIGINT)
                     AND (token.listing_end_date IS NULL OR token.listing_end_date >= (EXTRACT(EPOCH FROM NOW())::BIGINT))
                 ) * 100 / NULLIF(
                     (
                         SELECT COUNT(*)
                         FROM token
                         WHERE token.contract_address = $1
                         AND token.chain_id = contract.chain_id
                     ), 0
                 ) AS listed_percentage,
                 (
                      SELECT COUNT(*)
                      FROM token
                      WHERE token.contract_address = contract.contract_address
                      AND token.chain_id = contract.chain_id
                  ) AS token_count,
                 (
                   SELECT COUNT(DISTINCT current_owner)
                   FROM token
                   WHERE token.contract_address = contract.contract_address
                   AND token.chain_id = contract.chain_id
                ) AS owner_count,
                (
                     SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sell'
                ) AS total_volume,
                (
                     SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = $1
                     AND token_event.chain_id = contract.chain_id
                     AND token_event.event_type = 'Sell'
                 ) AS total_sales,
             contract_symbol
             FROM contract
             WHERE contract.contract_address = $1
             AND contract.chain_id = $2
             "#,
             contract_address,
             chain_id
         )
         .fetch_one(self)
         .await?;

        Ok(collection_data)
    }

    async fn get_tokens_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        page: i64,
        items_per_page: i64,
        buy_now: bool,
        sort: &str,
        direction: &str,
    ) -> Result<(Vec<TokenData>, bool, i64), Error> {
        let total_token_count = sqlx::query!(
            "
                SELECT COUNT(*)
                FROM token
                WHERE token.contract_address = $1
                  AND token.chain_id = $2
                ",
            contract_address,
            chain_id
        )
        .fetch_one(self)
        .await?;

        let token_count = total_token_count.count.unwrap_or(0);

        let total_count = sqlx::query!(
                "
                SELECT COUNT(*)
                FROM token
                WHERE token.contract_address = $1
                AND (
                    $2 = false OR
                    (EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN listing_start_date AND listing_end_date)
                )
                AND token.chain_id = $3
                ",
                contract_address,
                buy_now,
                chain_id
            )
            .fetch_one(self)
            .await?;

        let count = total_count.count.unwrap_or(0);

        let tokens_data: Vec<TokenData> = sqlx::query_as!(
               TokenData,
               "
               SELECT
                   token.contract_address as contract,
                   token.token_id,
                   hex_to_decimal(token.last_price) as last_price,
                   (
                      SELECT (((hex_to_decimal(token.listing_start_amount)) - MIN(hex_to_decimal(t1.listing_start_amount))) / MIN(hex_to_decimal(t1.listing_start_amount))) * 100
                      FROM token as t1
                      WHERE t1.contract_address = token.contract_address
                   ) as floor_difference,
                   token.listing_timestamp as listed_at,
                   token.current_owner as owner,
                   token.block_timestamp as minted_at,
                   token.updated_timestamp as updated_at,
                   hex_to_decimal(token.listing_start_amount) as price,
                   token.metadata as metadata
               FROM token
               WHERE token.contract_address = $3
                 AND token.chain_id = $4
               AND (
                   $5 = false OR
                   (EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN listing_start_date AND listing_end_date)
               )
               ORDER BY
               CASE
                  WHEN EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN token.listing_start_date AND token.listing_end_date THEN 1
                  ELSE 2
               END,
               CASE
                   WHEN $6 = 'price' THEN
                       CASE WHEN $7 = 'asc' THEN token.listing_start_amount
                            ELSE NULL
                       END
                   ELSE NULL
               END ASC,
               CASE
                   WHEN $6 = 'price' THEN
                       CASE WHEN $7 = 'desc' THEN token.listing_start_amount
                            ELSE NULL
                       END
                   ELSE NULL
               END DESC,
               CAST(token.token_id AS INTEGER)
           LIMIT $1 OFFSET $2",
               items_per_page,
               (page - 1) * items_per_page,
               contract_address,
               chain_id,
               buy_now,
               sort,
               direction,
           )
           .fetch_all(self)
           .await?;

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((tokens_data, has_next_page, token_count))
    }

    async fn get_tokens_portfolio_data(
        &self,
        user_address: &str,
        page: i64,
        items_per_page: i64,
        buy_now: bool,
        sort: &str,
        direction: &str,
        collection: &str,
    ) -> Result<(Vec<TokenPortfolioData>, bool, i64), Error> {
        let offset = (page - 1) * items_per_page;

        let total_token_count = sqlx::query!(
            "
                SELECT COUNT(*)
                FROM token
                WHERE token.current_owner = $1
                ",
            user_address
        )
        .fetch_one(self)
        .await?;

        let token_count = total_token_count.count.unwrap_or(0);

        let collection_filter = if collection.is_empty() {
            String::from("")
        } else {
            format!("AND token.contract_address = '{}'", collection)
        };

        let total_count_query = format!(
            "
            SELECT COUNT(*)
            FROM token
            WHERE token.current_owner = $1
            AND (
                $2 = false OR
                (EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN listing_start_date AND listing_end_date)
            )
            {}
            ",
            collection_filter
        );
        let total_count = sqlx::query(&total_count_query)
            .bind(user_address)
            .bind(buy_now)
            .fetch_one(self)
            .await?;

        let count: i64 = total_count.try_get(0).unwrap_or(0);

        let tokens_data_query = format!(
            "
            SELECT
                token.contract_address as contract,
                token.token_id,
                CAST(token.listing_start_amount AS NUMERIC) as list_price,
                (
                    SELECT MAX(CAST(offer_amount AS NUMERIC))
                    FROM token_offer
                    WHERE token_offer.token_id = token.token_id
                    AND (
                        EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN token_offer.start_date AND token_offer.end_date
                    )
                ) as best_offer,
                (
                    SELECT MIN(CAST(listing_start_amount AS NUMERIC))
                    FROM token
                    WHERE token.contract_address = $3
                ) as floor,
                token.held_timestamp as received_at,
                token.metadata as metadata
            FROM token
            WHERE token.current_owner = $3
            AND (
                $4 = false OR
                (EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN listing_start_date AND listing_end_date)
            )
            {}
            ORDER BY
            CASE
                WHEN EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) BETWEEN token.listing_start_date AND token.listing_end_date THEN 1
                ELSE 2
            END,
            CASE
                WHEN $5 = 'price' THEN
                    CASE WHEN $6 = 'asc' THEN token.listing_start_amount
                         ELSE NULL
                    END
                ELSE NULL
            END ASC,
            CASE
                WHEN $5 = 'price' THEN
                    CASE WHEN $6 = 'desc' THEN token.listing_start_amount
                         ELSE NULL
                    END
                ELSE NULL
            END DESC
            LIMIT $1 OFFSET $2
            ",
            collection_filter
        );

        let tokens_data: Vec<TokenPortfolioData> = sqlx::query_as(&tokens_data_query)
            .bind(items_per_page)
            .bind(offset)
            .bind(user_address)
            .bind(buy_now)
            .bind(sort)
            .bind(direction)
            .fetch_all(self)
            .await?;

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((tokens_data, has_next_page, token_count))
    }
}

#[cfg(test)]
pub struct MockDb;

#[cfg(test)]
#[async_trait]
impl DatabaseAccess for MockDb {
    async fn get_collections_data(
        &self,
        _page: i64,
        _items_per_page: i64,
        _time_range: i64,
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

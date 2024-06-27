use crate::models::collection::{CollectionData, CollectionPortfolioData};
use crate::models::token::{TokenData, TokenOneData, TokenPortfolioData};
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

    async fn get_token_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<TokenOneData, Error>;

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
                     hex_to_decimal(contract.floor_price) AS floor,
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
                         AND token.is_listed = true
                     ) AS listed_items,
                    (
                        SELECT COUNT(*)
                        FROM token
                        WHERE token.contract_address = contract.contract_address
                        AND token.chain_id = contract.chain_id
                        AND token.is_listed = true
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
                       AND  t1.is_listed = true
                  ) as user_listed_tokens,
                 contract.floor_price AS floor,
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
           GROUP BY contract.contract_address, contract.chain_id, token.current_owner, token.is_listed
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
                 contract.floor_price AS floor,
                 CAST(0 AS INTEGER) AS floor_7d_percentage,
                 CAST(0 AS INTEGER) AS volume_7d_eth,
                 contract.top_bid AS top_offer,
                 (
                     SELECT COUNT(*)
                     FROM token_event
                     WHERE token_event.contract_address = contract.contract_address
                     AND token_event.chain_id = contract.chain_id
                 ) AS sales_7d,
                 CAST(0 AS INTEGER) AS marketcap,
                 (
                     SELECT COUNT(*)
                     FROM token
                     WHERE token.contract_address = contract.contract_address
                     AND token.chain_id = contract.chain_id
                     AND token.is_listed = true
                 ) AS listed_items,
                 (
                     SELECT COUNT(*)
                     FROM token
                     WHERE token.contract_address = contract.contract_address
                     AND token.chain_id = contract.chain_id
                     AND token.is_listed = true
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
                     SELECT COUNT(*)
                     FROM (
                         SELECT current_owner
                         FROM token
                         WHERE contract_address = contract.contract_address
                           AND chain_id = contract.chain_id
                         GROUP BY current_owner
                     ) AS distinct_owners
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

    async fn get_token_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<TokenOneData, Error> {
        let token_data: TokenOneData = sqlx::query_as!(
            TokenOneData,
            "
                    SELECT
                        hex_to_decimal(token.listing_start_amount) as price,
                        hex_to_decimal(token.last_price) as last_price,
                        top_bid_amount as top_offer,
                        token.current_owner as owner,
                        c.contract_name as collection_name,
                        token.metadata as metadata
                    FROM token
                    INNER JOIN contract as c ON c.contract_address = token.contract_address
                        AND c.chain_id = token.chain_id
                    WHERE token.contract_address = $1
                      AND token.chain_id = $2
                      AND token.token_id = $3
                    ",
            contract_address,
            chain_id,
            token_id
        )
        .fetch_one(self)
        .await?;

        Ok(token_data)
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
                AND token.chain_id = $2
                AND (
                    $3 = false
                    OR token.is_listed = true
                )
                ",
            contract_address,
            chain_id,
            buy_now
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
                   CAST(0 as INTEGER) as floor_difference,
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
                    token.is_listed = true
              )
              ORDER BY
              CASE WHEN token.is_listed = true THEN 1 ELSE 2 END,
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
              CAST(token.token_id AS NUMERIC)
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
                is_listed = true
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
                hex_to_decimal(token.listing_start_amount) as list_price,
                hex_to_decimal(top_bid_amount) as best_offer,
                hex_to_decimal(c.floor_price) as floor,
                token.held_timestamp as received_at,
                token.metadata as metadata,
                c.contract_name as collection_name
            FROM token
            INNER JOIN contract as c ON c.contract_address = token.contract_address
                AND c.chain_id = token.chain_id
            WHERE token.current_owner = $3
            AND (
                $4 = false OR
                token.is_listed = true
            )
            {}
            ORDER BY
            CASE
                WHEN token.is_listed = true THEN 1
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
        }])
    }
}

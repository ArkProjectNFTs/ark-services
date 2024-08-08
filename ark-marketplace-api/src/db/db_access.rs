/// required to use map on stream
use futures_util::TryStreamExt;
use std::time::SystemTime;

use crate::models::collection::{
    CollectionActivityData, CollectionData, CollectionFloorPrice, CollectionPortfolioData,
    CollectionSearchData, OwnerData,
};
use crate::models::token::{
    Listing, TokenActivityData, TokenData, TokenDataListing, TokenEventType, TokenInformationData,
    TokenMarketData, TokenOfferOneDataDB, TokenOneData, TokenPortfolioData, TopOffer,
};
use crate::utils::db_utils::event_type_list;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde_json::Value as JsonValue;
use sqlx::Error;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Row;

const LISTING_TYPE_AUCTION_STR: &str = "Auction";

#[derive(FromRow)]
struct Count {
    total: i64,
}

#[derive(FromRow)]
struct TokenDataDB {
    pub collection_address: Option<String>,
    pub token_id: Option<String>,
    pub last_price: Option<BigDecimal>,
    pub floor_difference: Option<i32>,
    pub listed_at: Option<i64>,
    pub is_listed: Option<bool>,
    pub listing_type: Option<String>,
    pub price: Option<BigDecimal>,
    pub metadata: Option<JsonValue>,
    pub owner: Option<String>,
}

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
        sort: Option<String>,
        direction: Option<String>,
    ) -> Result<(Vec<TokenData>, bool, i64), Error>;

    async fn get_token_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<TokenInformationData, Error>;

    async fn get_token_marketdata(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<TokenMarketData, Error>;

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

    async fn get_token_offers_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
        page: i64,
        items_per_page: i64,
    ) -> Result<(Vec<TokenOfferOneDataDB>, bool, i64), Error>;

    async fn get_collections_data(
        &self,
        page: i64,
        items_per_page: i64,
        time_range: &str,
        user_address: Option<&str>,
    ) -> Result<Vec<CollectionData>, Error>;

    async fn search_collections_data(
        &self,
        query_search: Option<&str>,
        items: i64,
    ) -> Result<(Vec<CollectionSearchData>, Vec<OwnerData>), Error>;

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

    async fn get_collection_activity_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        page: i64,
        items_per_page: i64,
        direction: &str,
        types: &Option<Vec<TokenEventType>>,
    ) -> Result<(Vec<CollectionActivityData>, bool, i64), Error>;

    async fn get_collection_floor_price(
        &self,
        contract_address: &str,
        chain_id: &str,
    ) -> Result<CollectionFloorPrice, Error>;

    async fn get_token_activity_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
        page: i64,
        items_per_page: i64,
        direction: &str,
        types: &Option<Vec<TokenEventType>>,
    ) -> Result<(Vec<TokenActivityData>, bool, i64), Error>;

    async fn flush_all_data(&self) -> Result<u64, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn flush_all_data(&self) -> Result<u64, Error> {
        let mut total_rows_affected = 0;

        const TABLES: [&str; 2] = ["token_offer", "token_event"];

        for table in TABLES {
            let rows_affected = sqlx::query(format!("DELETE FROM {}", table).as_str())
                .execute(self)
                .await?
                .rows_affected();
            total_rows_affected += rows_affected;
        }

        let batch_size = 500;
        let mut offset = 0;

        loop {
            let select_query = format!(
                "
                SELECT token_id, contract_address, chain_id
                FROM token
                WHERE listing_start_amount IS NOT NULL
                   OR listing_end_amount IS NOT NULL
                   OR listing_start_date IS NOT NULL
                   OR listing_end_date IS NOT NULL
                   OR listing_currency_address IS NOT NULL
                   OR listing_orderhash IS NOT NULL
                   OR listing_timestamp IS NOT NULL
                   OR listing_currency_chain_id IS NOT NULL
                   OR listing_broker_id IS NOT NULL
                   OR last_price IS NOT NULL
                   OR top_bid_amount IS NOT NULL
                   OR top_bid_order_hash IS NOT NULL
                   OR top_bid_start_date IS NOT NULL
                   OR top_bid_end_date IS NOT NULL
                   OR top_bid_currency_address IS NOT NULL
                   OR top_bid_broker_id IS NOT NULL
                   OR has_bid IS NOT FALSE
                   OR buy_in_progress IS NOT FALSE
                   OR held_timestamp IS NOT NULL
                LIMIT {} OFFSET {}
                ",
                batch_size, offset
            );

            let tokens_array: [(String, String, String); 500] = sqlx::query_as(&select_query)
                .fetch_all(self)
                .await?
                .try_into()
                .expect("Expected exactly 500 tokens");

            if tokens_array.is_empty() {
                break;
            }

            for (token_id, contract_address, chain_id) in tokens_array.iter() {
                let update_query = "
                    UPDATE token
                    SET
                        listing_start_amount = null,
                        listing_end_amount = null,
                        listing_start_date = null,
                        listing_end_date = null,
                        listing_currency_address = null,
                        listing_orderhash = null,
                        listing_timestamp = null,
                        listing_currency_chain_id = null,
                        listing_broker_id = null,
                        last_price = null,
                        top_bid_amount = NULL,
                        top_bid_order_hash = NULL,
                        top_bid_start_date = NULL,
                        top_bid_end_date = NULL,
                        top_bid_currency_address = NULL,
                        top_bid_broker_id = NULL,
                        has_bid = false,
                        buy_in_progress = false,
                        held_timestamp = null
                    WHERE
                        contract_address = $1
                        AND chain_id = $2
                        AND token_id = $3
                ";

                sqlx::query(update_query)
                    .bind(contract_address)
                    .bind(chain_id)
                    .bind(token_id)
                    .execute(self)
                    .await?;
            }

            offset += batch_size;
        }

        Ok(total_rows_affected)
    }

    async fn search_collections_data(
        &self,
        query_search: Option<&str>,
        items: i64,
    ) -> Result<(Vec<CollectionSearchData>, Vec<OwnerData>), Error> {
        let mut collections = Vec::new();
        let mut accounts = Vec::new();

        if let Some(ref cleaned) = query_search {
            if !cleaned.is_empty() {
                // Search for owner matching the query
                let account_query = "SELECT distinct token.chain_id, current_owner as owner FROM token WHERE current_owner = $1 OR current_owner = $2";
                accounts = sqlx::query_as::<_, OwnerData>(account_query)
                    .bind(format!("0x0{}", cleaned))
                    .bind(format!("0x00{}", cleaned))
                    .fetch_all(self)
                    .await?;

                // Search for collections matching the query
                let collection_query = format!(
                    "SELECT
                         contract.contract_address as address,
                         contract_image AS image,
                         contract_name AS name,
                         token_count AS token_count,
                         is_verified AS is_verified
                     FROM
                         contract
                     WHERE contract_name ILIKE '%{}%' OR contract.contract_address ILIKE '%{}%'
                     ORDER BY token_count desc, is_verified desc, contract_name
                     LIMIT {}
                     ",
                    cleaned, cleaned, items
                );

                collections =
                    sqlx::query_as::<sqlx::Postgres, CollectionSearchData>(&collection_query)
                        .fetch_all(self)
                        .await?;
            }
        }

        Ok((collections, accounts))
    }

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
                    contract_name AS name,
                    floor_price AS floor,
                    COALESCE(
                        (
                            SELECT
                                (contract.floor_price - fc.floor) / fc.floor * 100
                            FROM
                                floor_collection fc
                            WHERE
                                fc.contract_address = contract.contract_address
                                AND fc.chain_id = contract.chain_id
                                AND fc.timestamp >= (CURRENT_DATE - INTERVAL '7 days')
                            ORDER BY
                                fc.timestamp ASC
                            LIMIT 1
                        ),
                        0
                    ) AS floor_7d_percentage,
                    volume_7d_eth,
                    top_bid as top_offer
                    sales_7d,
                    marketcap,
                    token_listed_count,
                    listed_percentage,
                    token_count,
                    owner_count,
                    total_volume
                    total_sales
                    FROM
                     contract
                     INNER JOIN token ON contract.contract_address = token.contract_address AND contract.chain_id = token.chain_id
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
                println!("Query error : {}", err);
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
                INNER JOIN token ON contract.contract_address = token.contract_address AND contract.chain_id = token.chain_id
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
                 contract_name AS name,
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
                       AND  t1.listing_start_amount is not null
                  ) as user_listed_tokens,
                 contract.floor_price AS floor,
                 contract.token_count
                FROM
                 contract
                 INNER JOIN token ON contract.contract_address = token.contract_address AND contract.chain_id = token.chain_id
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
                 contract_name AS name,
                 contract.floor_price AS floor,
                 volume_7d_eth,
                 contract.top_bid AS top_offer,
                 sales_7d,
                 marketcap,
                 token_listed_count AS listed_items,
                 listed_percentage,
                 token_count,
                 owner_count,
                 total_volume,
                 total_sales,
                 floor_7d_percentage
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

    async fn get_collection_activity_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        page: i64,
        items_per_page: i64,
        direction: &str,
        types: &Option<Vec<TokenEventType>>,
    ) -> Result<(Vec<CollectionActivityData>, bool, i64), Error> {
        let offset = (page - 1) * items_per_page;
        let types_filter = match types {
            None => String::from(""),
            Some(values) => {
                format!("AND te.event_type IN ({})", event_type_list(values))
            }
        };
        let common_sql_query = format!(
            "
                FROM token_event te
                LEFT JOIN token_offer ON te.order_hash = token_offer.order_hash
                LEFT JOIN token ON te.token_id = token.token_id and te.contract_address = token.contract_address and te.chain_id = token.chain_id
                LEFT JOIN contract ON te.contract_address = contract.contract_address and te.chain_id = contract.chain_id
                WHERE te.contract_address = $1
                    AND te.chain_id = $2
                    {}
            ",
            types_filter
        );

        let count_sql_query = format!(
            "
            SELECT COUNT(*) AS total
            FROM token_event te
            WHERE te.contract_address = $1
                AND te.chain_id = $2
                {}
            ",
            types_filter
        );

        let total_count: Count = sqlx::query_as(&count_sql_query)
            .bind(contract_address)
            .bind(chain_id)
            .fetch_one(self)
            .await?;
        let count = total_count.total;

        let price_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN hex_to_decimal(token_offer.offer_amount)
                ELSE hex_to_decimal(te.amount)
            END AS price
            ",
            event_type_list(&[TokenEventType::Fulfill, TokenEventType::Executed])
        );

        let from_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN token_offer.from_address
                ELSE te.from_address
            END AS from
            ",
            event_type_list(&[TokenEventType::Fulfill, TokenEventType::Executed])
        );

        let to_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN token_offer.to_address
                ELSE te.to_address
            END AS to
            ",
            event_type_list(&[TokenEventType::Fulfill, TokenEventType::Executed])
        );

        let activity_sql_query = format!(
            "
            SELECT
                CASE
                    WHEN te.event_type = 'Executed' THEN 'Sale'
                    ELSE te.event_type
                END AS activity_type,
                te.block_timestamp AS time_stamp,
                te.transaction_hash,
                te.token_id,
                token.metadata as token_metadata,
                contract.contract_name as name,
                contract.is_verified,
                {},
                {},
                {}
            {}
            ORDER BY te.block_timestamp {}
            LIMIT {} OFFSET {}
            ",
            price_select_part,
            from_select_part,
            to_select_part,
            common_sql_query,
            direction,
            items_per_page,
            offset,
        );

        let collection_activity_data: Vec<CollectionActivityData> =
            sqlx::query_as(&activity_sql_query)
                .bind(contract_address)
                .bind(chain_id)
                .fetch_all(self)
                .await?;

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((collection_activity_data, has_next_page, count))
    }

    async fn get_collection_floor_price(
        &self,
        contract_address: &str,
        chain_id: &str,
    ) -> Result<CollectionFloorPrice, Error> {
        let floor_price_query = "SELECT floor_price AS value FROM contract WHERE contract_address = $1 AND chain_id = $2";
        let floor_price = sqlx::query_as(floor_price_query)
            .bind(contract_address)
            .bind(chain_id)
            .fetch_one(self)
            .await?;

        Ok(floor_price)
    }

    async fn get_token_marketdata(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<TokenMarketData, Error> {
        // Fetch TokenOneData
        let token_data: TokenOneData = sqlx::query_as!(
            TokenOneData,
            "
                SELECT
                    token.current_owner as owner,
                    hex_to_decimal(token.last_price) as floor,
                    token.listing_timestamp as created_timestamp,
                    token.updated_timestamp as updated_timestamp,
                    (token.listing_start_amount IS NOT NULL) as is_listed,
                    has_bid as has_offer,
                    token.buy_in_progress as buy_in_progress,
                    hex_to_decimal(token.last_price) as last_price
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

        // Fetch TopOffer
        let top_offer: TopOffer = sqlx::query_as!(
            TopOffer,
            "
                SELECT
                    top_bid_order_hash as order_hash,
                    top_bid_amount as amount,
                    top_bid_start_date as start_date,
                    top_bid_end_date as end_date,
                    top_bid_currency_address as currency_address
                FROM token
                WHERE token.token_id = $1
                AND token.contract_address = $2
            ",
            token_id,
            contract_address
        )
        .fetch_one(self)
        .await
        .unwrap_or(TopOffer {
            order_hash: Some("".to_string()),
            amount: None,
            start_date: None,
            end_date: None,
            currency_address: Some("".to_string()),
        });

        // Fetch Listing
        let listing: Listing = sqlx::query_as!(
            Listing,
            "
                SELECT
                    (t.listing_type = 'Auction') as is_auction,
                    listing_orderhash as order_hash,
                    listing_start_amount as start_amount,
                    listing_end_amount as end_amount,
                    listing_start_date as start_date,
                    listing_end_date as end_date,
                    listing_currency_address as currency_address
                FROM token t
                WHERE t.token_id = $1
                AND t.contract_address = $2
                LIMIT 1
            ",
            token_id,
            contract_address
        )
        .fetch_one(self)
        .await
        .unwrap_or(Listing {
            is_auction: Some(false),
            order_hash: Some("".to_string()),
            start_amount: None,
            end_amount: None,
            start_date: None,
            end_date: None,
            currency_address: Some("".to_string()),
        });

        Ok(TokenMarketData {
            owner: token_data.owner,
            floor: token_data.floor,
            created_timestamp: token_data.created_timestamp,
            updated_timestamp: token_data.updated_timestamp,
            is_listed: token_data.is_listed,
            has_offer: token_data.has_offer,
            buy_in_progress: token_data.buy_in_progress,
            top_offer: Some(top_offer),
            listing: Some(listing),
            last_price: token_data.last_price,
        })
    }

    async fn get_token_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<TokenInformationData, Error> {
        let token_data: TokenInformationData = sqlx::query_as!(
            TokenInformationData,
            "
                    SELECT
                        token_id,
                        token.contract_address as collection_address,
                        hex_to_decimal(token.listing_start_amount) as price,
                        hex_to_decimal(token.last_price) as last_price,
                        top_bid_amount as top_offer,
                        token.current_owner as owner,
                        c.contract_name as collection_name,
                        token.metadata as metadata,
                        c.contract_image as collection_image
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
        sort: Option<String>,
        direction: Option<String>,
    ) -> Result<(Vec<TokenData>, bool, i64), Error> {
        let sort_field = sort.as_deref().unwrap_or("price");
        let sort_direction = direction.as_deref().unwrap_or("asc");

        let order_by = match (sort_field, sort_direction) {
            ("price", "asc") => {
                "token.listing_start_amount ASC NULLS LAST, CAST(token.token_id AS NUMERIC)"
            }
            ("price", "desc") => {
                "token.listing_start_amount DESC NULLS FIRST, CAST(token.token_id AS NUMERIC)"
            }
            (_, "asc") => "CAST(token.token_id AS NUMERIC) ASC",
            (_, "desc") => "CAST(token.token_id AS NUMERIC) DESC",
            _ => "CAST(token.token_id AS NUMERIC) ASC", // Default case
        };
        /*let total_token_count = sqlx::query!(
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
        .await?;*/

        //let token_count = total_token_count.count.unwrap_or(0);
        let token_count = 0;

        /*let total_count = sqlx::query!(
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
        .await?;*/

        //let count = total_count.count.unwrap_or(0);
        let count = 0;

        let tokens_data_query = format!(
            "
               SELECT
                   token.contract_address as collection_address,
                   token.token_id,
                   hex_to_decimal(token.last_price) as last_price,
                   CAST(0 as INTEGER) as floor_difference,
                   token.listing_timestamp as listed_at,
                   (token.listing_start_amount IS NOT NULL) as is_listed,
                   token.listing_type as listing_type,
                   hex_to_decimal(token.listing_start_amount) as price,
                   token.metadata as metadata,
                   current_owner as owner
               FROM token
               WHERE token.contract_address = $1
                   AND token.chain_id = $2
                   AND ($3 = false OR token.listing_start_amount IS NOT NULL)
               ORDER BY {}
               LIMIT $4 OFFSET $5",
            order_by
        );

        let query = sqlx::query_as(&tokens_data_query)
            .bind(contract_address)
            .bind(chain_id)
            .bind(buy_now)
            .bind(items_per_page)
            .bind((page - 1) * items_per_page);

        let tokens_data: Vec<TokenData> = query
            .fetch(self)
            .map_ok(|token: TokenDataDB| TokenData {
                collection_address: token.collection_address,
                token_id: token.token_id,
                last_price: token.last_price,
                floor_difference: token.floor_difference,
                listed_at: token.listed_at,
                is_listed: token.is_listed,
                listing: token
                    .listing_type
                    .as_ref()
                    .map(|listing_type| TokenDataListing {
                        is_auction: Some(listing_type == LISTING_TYPE_AUCTION_STR),
                    }),
                metadata: token.metadata,
                price: token.price,
                owner: token.owner,
            })
            .try_collect()
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
                token.listing_start_amount IS NOT NULL
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
                token.contract_address as collection_address,
                token.token_id,
                hex_to_decimal(token.listing_start_amount) as list_price,
                top_bid_amount as best_offer,
                c.floor_price as floor,
                token.held_timestamp as received_at,
                token.metadata as metadata,
                c.contract_name as collection_name
            FROM token
            INNER JOIN contract as c ON c.contract_address = token.contract_address
                AND c.chain_id = token.chain_id
            WHERE token.current_owner = $3
            AND (
                $4 = false OR
                token.listing_start_amount IS NOT NULL
            )
            {}
            ORDER BY listing_start_amount ASC NULLS LAST,
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

    async fn get_token_offers_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
        page: i64,
        items_per_page: i64,
    ) -> Result<(Vec<TokenOfferOneDataDB>, bool, i64), Error> {
        // FIXME: pagination assume that all offers used the same currency
        let offset = (page - 1) * items_per_page;
        let current_time: i64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => d.as_secs().try_into().unwrap(),
            Err(_) => 0,
        };
        let total_count = sqlx::query!(
            "SELECT COUNT(*)
            FROM token_offer
            WHERE token_offer.contract_address = $1
                AND token_offer.chain_id = $2
                AND token_offer.token_id = $3
                AND token_offer.status = 'PLACED'
                AND end_date > $4
            ",
            contract_address,
            chain_id,
            token_id,
            current_time
        )
        .fetch_one(self)
        .await?;

        let count = total_count.count.unwrap_or(0);

        let token_offers_data = sqlx::query_as!(
            TokenOfferOneDataDB,
            "SELECT
                token_offer_id AS offer_id,
                hex_to_decimal(offer_amount) AS amount,
                offer_maker AS source,
                end_date AS expire_at,
                order_hash as hash,
                currency_address
            FROM token_offer
            WHERE token_offer.contract_address = $1
                AND token_offer.chain_id = $2
                AND token_offer.token_id = $3
                AND token_offer.status = 'PLACED'
                AND end_date > $4
            ORDER BY amount DESC, expire_at ASC
            LIMIT $5 OFFSET $6
            ",
            contract_address,
            chain_id,
            token_id,
            current_time,
            items_per_page,
            offset
        )
        .fetch_all(self)
        .await?;

        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((token_offers_data, has_next_page, count))
    }

    async fn get_token_activity_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
        page: i64,
        items_per_page: i64,
        direction: &str,
        types: &Option<Vec<TokenEventType>>,
    ) -> Result<(Vec<TokenActivityData>, bool, i64), Error> {
        let offset = (page - 1) * items_per_page;

        let token_event_with_previous_cte_part = format!(
            "
            WITH temporary_event_with_previous AS (
            (
                SELECT
                    *,
                    -- LAG() function is a window function that provides access to a row at a specified physical offset which comes before the current row.
                    -- Here we want to retrieve the previous event type for given order_hash
                    LAG(event_type) OVER (PARTITION BY order_hash ORDER BY block_timestamp) AS previous_event_type
                    FROM token_event WHERE contract_address = $1 AND chain_id = $2 AND token_id = $3
                )
            ),
            token_event_with_previous AS (
                SELECT
                    *,
                    -- Create new event type if needed
                    CASE
                        WHEN event_type = '{executed_type}' THEN '{sale_type}'
                        WHEN event_type = '{cancelled_type}' AND previous_event_type = '{listing_type}' THEN '{listing_cancelled_type}'
                        WHEN event_type = '{cancelled_type}' AND previous_event_type = '{auction_type}' THEN '{auction_cancelled_type}'
                        WHEN event_type = '{cancelled_type}' AND previous_event_type = '{offer_type}' THEN '{offer_cancelled_type}'
                        ELSE event_type
                    END AS new_event_type
                    FROM temporary_event_with_previous
                    WHERE contract_address = $1 AND chain_id = $2 AND token_id = $3
                )
            ",
            executed_type = TokenEventType::Executed.to_db_string(),
            sale_type = TokenEventType::Sale.to_db_string(),
            cancelled_type = TokenEventType::Cancelled.to_db_string(),
            listing_type = TokenEventType::Listing.to_db_string(),
            auction_type = TokenEventType::Auction.to_db_string(),
            offer_type = TokenEventType::Offer.to_db_string(),
            listing_cancelled_type = TokenEventType::ListingCancelled.to_db_string(),
            auction_cancelled_type = TokenEventType::AuctionCancelled.to_db_string(),
            offer_cancelled_type = TokenEventType::OfferCancelled.to_db_string(),
        );

        let types_filter = match types {
            None => String::from(""),
            Some(values) => {
                format!("AND te.new_event_type IN ({})", event_type_list(values))
            }
        };

        let common_where_part = format!(
            "
                FROM token_event_with_previous as te
                LEFT JOIN token_offer ON te.order_hash = token_offer.order_hash
                LEFT JOIN token ON te.token_id = token.token_id and te.contract_address = token.contract_address and te.chain_id = token.chain_id
                LEFT JOIN contract ON te.contract_address = contract.contract_address and te.chain_id = contract.chain_id
                WHERE te.contract_address = $1
                    AND te.chain_id = $2
                    AND te.token_id = $3
                    {}
                    AND te.new_event_type NOT IN ({})
            ",
            types_filter,
            event_type_list(&[TokenEventType::Fulfill])
        );

        let count_sql_query = format!(
            "
            {}
            SELECT COUNT(*) AS total
            {}
            ",
            token_event_with_previous_cte_part, common_where_part,
        );

        let total_count: Count = sqlx::query_as(&count_sql_query)
            .bind(contract_address)
            .bind(chain_id)
            .bind(token_id)
            .fetch_one(self)
            .await?;
        let count = total_count.total;

        let price_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN hex_to_decimal(token_offer.offer_amount)
                ELSE hex_to_decimal(te.amount)
            END AS price
            ",
            event_type_list(&[TokenEventType::Fulfill, TokenEventType::Executed])
        );

        let from_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN token_offer.from_address
                ELSE te.from_address
            END AS from
            ",
            event_type_list(&[TokenEventType::Fulfill, TokenEventType::Executed])
        );

        let to_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN token_offer.to_address
                ELSE te.to_address
            END AS to
            ",
            event_type_list(&[TokenEventType::Fulfill, TokenEventType::Executed])
        );

        let result_ordering = format!(
            "
            ORDER BY te.block_timestamp {direction}
            LIMIT {limit} OFFSET {offset}
            ",
            direction = direction,
            limit = items_per_page,
            offset = offset,
        );

        let activity_sql_query = format!(
            "
            {token_event_with_previous_cte}
            SELECT
                te.block_timestamp AS time_stamp,
                te.transaction_hash,
                te.new_event_type AS activity_type,
                token.metadata,
                contract.contract_name as collection_name,
                contract.is_verified as collection_is_verified,
                {price_select},
                {from_select},
                {to_select}
            {common_where}
            {result_ordering}
            ",
            token_event_with_previous_cte = token_event_with_previous_cte_part,
            common_where = common_where_part,
            price_select = price_select_part,
            from_select = from_select_part,
            to_select = to_select_part,
        );

        let token_activity_data: Vec<TokenActivityData> = sqlx::query_as(&activity_sql_query)
            .bind(contract_address)
            .bind(chain_id)
            .bind(token_id)
            .fetch_all(self)
            .await?;

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((token_activity_data, has_next_page, count))
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

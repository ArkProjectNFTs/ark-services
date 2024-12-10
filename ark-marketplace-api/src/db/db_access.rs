use crate::models::collection::{
    CollectionActivityData, CollectionActivityDataDB, CollectionData, CollectionFloorPrice,
    CollectionFullData, CollectionPortfolioData, CollectionSearchData, OwnerData,
};
use crate::models::default::Currency;
use crate::models::token::{
    Listing, ListingRaw, TokenActivityData, TokenActivityDataDB, TokenData, TokenDataListing,
    TokenEventType, TokenInformationData, TokenMarketData, TokenOfferOneDataDB, TokenOneData,
    TokenPortfolioData, TopOffer, TopOfferQueryResult,
};
use crate::utils::db_utils::event_type_list;
use crate::utils::sql_utils::{generate_order_by_clause, generate_order_by_clause_collections};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use serde_json::Value as JsonValue;
use sqlx::Error;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::HashMap;
use std::time::SystemTime;

pub const LISTING_TYPE_AUCTION_STR: &str = "Auction";

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
    pub currency_address: Option<String>,
    pub buy_in_progress: Option<bool>,
}

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait DatabaseAccess: Send + Sync {
    async fn refresh_token_metadata(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<(), Error>;

    async fn get_token_top_offer(
        &self,
        token_id: &str,
        contract_address: &str,
        currencies: &[Currency],
    ) -> Result<Option<TopOffer>, Error>;

    async fn get_tokens_data(
        &self,
        contract_address: &str,
        chain_id: &str,
        page: i64,
        items_per_page: i64,
        buy_now: bool,
        sort: Option<String>,
        direction: Option<String>,
        sort_value: Option<String>,
        token_ids: Option<Vec<String>>,
        token_id: Option<String>,
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
        sort: &str,
        direction: &str,
    ) -> Result<(Vec<CollectionFullData>, bool, i64), Error>;

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

    async fn get_currency(
        &self,
        currencies: Vec<Currency>,
        currency_contract_address: Option<String>,
    ) -> Currency;

    async fn get_currencies(&self) -> Result<Vec<Currency>, Error>;

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
    async fn refresh_token_metadata(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<(), Error> {
        let query = "
            UPDATE token
            SET metadata_status = 'TO_REFRESH', updated_timestamp = EXTRACT(EPOCH FROM NOW())
            WHERE contract_address = $1 AND chain_id = $2 AND token_id = $3
        ";

        sqlx::query(query)
            .bind(contract_address)
            .bind(chain_id)
            .bind(token_id)
            .execute(self)
            .await?;

        Ok(())
    }

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
        sort: &str,
        direction: &str,
    ) -> Result<(Vec<CollectionFullData>, bool, i64), Error> {
        const MIN_COLLECTION_COUNT: usize = 100;
        let total_count = sqlx::query!("SELECT count(*) FROM contract")
            .fetch_one(self)
            .await?;

        let count = total_count.count.unwrap_or(0);

        const ALLOWED_SORTS: &[&str] = &[
            "floor_price",
            "floor_percentage",
            "volume",
            "top_bid",
            "number_of_sales",
            "marketcap",
            "listed",
        ];
        const ALLOWED_DIRECTIONS: &[&str] = &["asc", "desc"];
        if !ALLOWED_SORTS.contains(&sort) || !ALLOWED_DIRECTIONS.contains(&direction) {
            tracing::error!("get_collections_data: Invalid sort or direction");
        }
        let order_by_clause = generate_order_by_clause_collections(sort, direction);

        let contract_timestamp_clause: String = if time_range.is_empty() {
            String::new()
        } else {
            format!(" AND contract_marketdata.timerange = '{}'", time_range)
        };

        let sql_query = format!(
            "SELECT
                    contract.contract_address as address,
                    contract_image AS image,
                    contract_name AS name,
                    floor_price AS floor,
                    contract_marketdata.floor_percentage as floor_percentage,
                    contract_marketdata.volume as volume,
                    top_bid as top_offer,
                    contract_marketdata.number_of_sales as sales,
                    marketcap,
                    token_listed_count AS listed_items,
                    listed_percentage,
                    token_count,
                    owner_count,
                    total_volume,
                    total_sales,
                    is_verified
                    FROM
                     contract
                     INNER JOIN contract_marketdata on contract.contract_address = contract_marketdata.contract_address and contract.chain_id = contract_marketdata.chain_id {}
                     WHERE contract_marketdata.volume > 0
               GROUP BY contract.contract_address, contract.chain_id, floor_percentage, volume, sales
               {}
               LIMIT {} OFFSET {}
               ",
            contract_timestamp_clause,
            order_by_clause,
            items_per_page,
            (page - 1) * items_per_page,
        );
        let mut collection_data = sqlx::query_as::<sqlx::Postgres, CollectionFullData>(&sql_query)
            .fetch_all(self)
            .await?;

        if collection_data.len() < MIN_COLLECTION_COUNT {
            let missing_count = MIN_COLLECTION_COUNT - collection_data.len();

            // Get the contract addresses already included in collection_data
            let existing_addresses: Vec<&str> = collection_data
                .iter()
                .map(|data| data.address.as_str())
                .collect();

            let addresses_placeholder = existing_addresses
                .iter()
                .map(|address| format!("'{}'", address))
                .collect::<Vec<_>>()
                .join(", ");

            let additional_sql_query = format!(
                "SELECT
                    contract.contract_address as address,
                    contract_image AS image,
                    contract_name AS name,
                    floor_price AS floor,
                    contract_marketdata.floor_percentage as floor_percentage,
                    contract_marketdata.volume as volume,
                    top_bid as top_offer,
                    contract_marketdata.number_of_sales as sales,
                    marketcap,
                    token_listed_count AS listed_items,
                    listed_percentage,
                    token_count,
                    owner_count,
                    total_volume,
                    total_sales,
                    is_verified
                    FROM
                     contract
                     INNER JOIN contract_marketdata on contract.contract_address = contract_marketdata.contract_address and contract.chain_id = contract_marketdata.chain_id
                            AND contract_marketdata.timerange = '30d'
                            AND contract_marketdata.volume > 0
                WHERE contract.contract_address NOT IN ({})
                GROUP BY contract.contract_address, contract.chain_id, floor_percentage, volume, sales
                ORDER BY contract_marketdata.volume DESC NULLS LAST
               LIMIT {}
               ",
                addresses_placeholder,
                missing_count
            );

            let additional_collections: Vec<CollectionFullData> =
                sqlx::query_as(&additional_sql_query)
                    .fetch_all(self)
                    .await?;

            collection_data.extend(additional_collections);

            // if we still need some collection we collect use number of owner
            if collection_data.len() < MIN_COLLECTION_COUNT {
                let missing_count = MIN_COLLECTION_COUNT - collection_data.len();

                // Get the contract addresses already included in collection_data
                let existing_addresses: Vec<&str> = collection_data
                    .iter()
                    .map(|data| data.address.as_str())
                    .collect();

                let addresses_placeholder = existing_addresses
                    .iter()
                    .map(|address| format!("'{}'", address))
                    .collect::<Vec<_>>()
                    .join(", ");

                let additional_sql_query = format!(
                    "SELECT
                    contract.contract_address as address,
                    contract_image AS image,
                    contract_name AS name,
                    floor_price AS floor,
                    CAST(0 AS BIGINT) AS floor_percentage,
                    CAST(0 AS BIGINT) AS volume,
                    top_bid as top_offer,
                    CAST(0 AS BIGINT) AS sales,
                    marketcap,
                    token_listed_count AS listed_items,
                    listed_percentage,
                    token_count,
                    owner_count,
                    total_volume,
                    total_sales,
                    is_verified
                    FROM
                     contract
                WHERE contract.contract_address NOT IN ({})
                  AND contract.owner_count > 0
                GROUP BY contract.contract_address, contract.chain_id
                ORDER BY owner_count desc
               LIMIT {}
               ",
                    addresses_placeholder, missing_count
                );

                let additional_collections: Vec<CollectionFullData> =
                    sqlx::query_as(&additional_sql_query)
                        .fetch_all(self)
                        .await?;

                collection_data.extend(additional_collections);
            }
        }

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((collection_data, has_next_page, count))
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
                 floor_7d_percentage,
                 is_verified,
                 deployed_timestamp,
                 website,
                 twitter,
                 discord,
                 description
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
            event_type_list(&[TokenEventType::Fulfill])
        );

        let from_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN token_offer.from_address
                ELSE te.from_address
            END AS from
            ",
            event_type_list(&[TokenEventType::Fulfill])
        );

        let to_select_part = format!(
            "
            CASE
                WHEN te.event_type in ({}) THEN token_offer.to_address
                ELSE te.to_address
            END AS to
            ",
            event_type_list(&[TokenEventType::Fulfill])
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
                te.currency_address,
                token.metadata as token_metadata,
                contract.contract_name as name,
                contract.is_verified,
                contract.contract_address as address,
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

        let collection_activity_data_db: Vec<CollectionActivityDataDB> =
            sqlx::query_as(&activity_sql_query)
                .bind(contract_address)
                .bind(chain_id)
                .fetch_all(self)
                .await?;

        let currencies: Vec<Currency> = sqlx::query_as!(
            Currency,
            r#"SELECT currency_address as contract, symbol, decimals FROM public.currency_mapping"#
        )
        .fetch_all(self)
        .await?;

        let collection_activity_data: Vec<CollectionActivityData> = collection_activity_data_db
            .into_iter()
            .map(|sale| {
                let currency = currencies
                    .iter()
                    .find(|c| c.contract == sale.currency_address)
                    .cloned();
                CollectionActivityData {
                    activity_type: sale.activity_type,
                    price: sale.price,
                    from: sale.from,
                    to: sale.to,
                    time_stamp: sale.time_stamp,
                    transaction_hash: sale.transaction_hash,
                    token_id: sale.token_id,
                    token_metadata: sale.token_metadata,
                    name: sale.name,
                    address: sale.address,
                    is_verified: sale.is_verified,
                    currency,
                }
            })
            .collect();

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

    async fn get_currencies(&self) -> Result<Vec<Currency>, Error> {
        let currencies: Vec<Currency> = sqlx::query_as!(
            Currency,
            r#"SELECT currency_address as contract, symbol, decimals FROM public.currency_mapping"#
        )
        .fetch_all(self)
        .await?;

        Ok(currencies)
    }

    async fn get_currency(
        &self,
        currencies: Vec<Currency>,
        currency_contract_address: Option<String>,
    ) -> Currency {
        let address = match currency_contract_address {
            Some(addr) => addr,
            None => return Currency::default(),
        };

        currencies
            .iter()
            .find(|c| c.contract.as_deref() == Some(&address))
            .or_else(|| {
                currencies
                    .iter()
                    .find(|c| c.symbol.as_deref() == Some("ETH"))
            })
            .cloned()
            .unwrap_or_else(Currency::default)
    }

    async fn get_token_top_offer(
        &self,
        token_id: &str,
        contract_address: &str,
        currencies: &[Currency],
    ) -> Result<Option<TopOffer>, Error> {
        let top_offer_result = sqlx::query_as!(
            TopOfferQueryResult,
            r#"
            SELECT
                top_bid_order_hash as order_hash,
                top_bid_amount as amount,
                top_bid_start_date as start_date,
                top_bid_end_date as end_date,
                top_bid_currency_address as currency_address
            FROM token
            WHERE token.token_id = $1
            AND token.contract_address = $2
            "#,
            token_id,
            contract_address
        )
        .fetch_optional(self)
        .await?;

        if let Some(result) = top_offer_result {
            if result.order_hash.is_some() {
                let currency = self
                    .get_currency(currencies.to_vec(), result.currency_address)
                    .await;
                Ok(Some(TopOffer {
                    order_hash: result.order_hash,
                    amount: result.amount,
                    start_date: result.start_date,
                    end_date: result.end_date,
                    currency,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn get_token_marketdata(
        &self,
        contract_address: &str,
        chain_id: &str,
        token_id: &str,
    ) -> Result<TokenMarketData, Error> {
        let currencies = self.get_currencies().await?;

        // Fetch TokenOneData
        let token_data: TokenOneData = sqlx::query_as!(
            TokenOneData,
            "
                SELECT
                    token.current_owner as owner,
                    token.listing_currency_address as listing_currency_address,
                    token.listing_currency_chain_id as listing_currency_chain_id,
                    c.floor_price as floor,
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

        let top_offer = self
            .get_token_top_offer(token_id, contract_address, &currencies)
            .await?;

        let listing_currency = self
            .get_currency(currencies.clone(), token_data.listing_currency_address)
            .await;

        // Fetch Listing
        let listing: ListingRaw = sqlx::query_as!(
            ListingRaw,
            "
                SELECT
                    (t.listing_type = 'Auction') as is_auction,
                    listing_orderhash as order_hash,
                    listing_start_amount as start_amount,
                    listing_end_amount as end_amount,
                    listing_start_date as start_date,
                    listing_end_date as end_date
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
        .unwrap_or(ListingRaw {
            is_auction: Some(false),
            order_hash: Some("".to_string()),
            start_amount: None,
            end_amount: None,
            start_date: None,
            end_date: None,
        });

        Ok(TokenMarketData {
            owner: token_data.owner,
            floor: token_data.floor,
            created_timestamp: token_data.created_timestamp,
            updated_timestamp: token_data.updated_timestamp,
            is_listed: token_data.is_listed,
            has_offer: token_data.has_offer,
            buy_in_progress: token_data.buy_in_progress,
            top_offer,
            listing: Some(Listing {
                is_auction: listing.is_auction,
                order_hash: listing.order_hash,
                start_amount: listing.start_amount,
                end_amount: listing.end_amount,
                start_date: listing.start_date,
                end_date: listing.end_date,
                currency: listing_currency,
            }),
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
                        c.contract_image as collection_image,
                        metadata_updated_at,
                        metadata_status
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
        sort_value: Option<String>,
        token_ids: Option<Vec<String>>,
        token_id: Option<String>,
    ) -> Result<(Vec<TokenData>, bool, i64), Error> {
        let sort_field = sort.as_deref().unwrap_or("price");
        let sort_direction = direction.as_deref().unwrap_or("asc");
        let order_by = generate_order_by_clause(sort_field, sort_direction, sort_value.as_deref());
        let count = 0;

        // Additional condition for token_id if it's provided
        let token_id_condition = if let Some(ref id) = token_id {
            format!("AND token.token_id LIKE '%{}%'", id)
        } else {
            String::new()
        };

        let (token_ids_condition, token_count) = match token_ids {
            Some(ids) if !ids.is_empty() => {
                let condition = format!(
                    "AND token.token_id IN ({})",
                    ids.iter()
                        .map(|id| format!("'{}'", id))
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                // calculate the token count
                let token_count_query = format!(
                    "
                    SELECT COUNT(*)
                    FROM token
                    WHERE token.contract_address = $1
                        AND token.chain_id = $2
                        AND ($3 = false OR (token.listing_start_amount IS NOT NULL AND token.listing_type != 'Auction'))
                        {} {}
                    ",
                    condition, token_id_condition
                );

                let token_count: i64 = sqlx::query_scalar(&token_count_query)
                    .bind(contract_address)
                    .bind(chain_id)
                    .bind(buy_now)
                    .fetch_one(self)
                    .await?;

                (condition, token_count)
            }
            Some(_) => {
                // If the token_ids is empty, no result
                let condition = "AND 1 = 0".to_string();
                (condition, 0)
            }
            None => {
                let condition = String::new();
                // get fields from contract table to calculate token count
                let contract_query = "
                    SELECT
                        token_count,
                        token_listed_count
                    FROM contract
                    WHERE contract_address = $1
                    AND chain_id = $2
                    "
                .to_string();

                let contract_data: (Option<i64>, Option<i64>) = sqlx::query_as(&contract_query)
                    .bind(contract_address)
                    .bind(chain_id)
                    .fetch_one(self)
                    .await?;

                let token_count = contract_data.0.unwrap_or(0);
                let token_listed_count = contract_data.1.unwrap_or(0);

                // if buy now is true, then token count is token_listed_count
                // else token count is token_count - token_listed_count
                let token_count = if buy_now {
                    token_listed_count
                } else {
                    token_count - token_listed_count
                };

                (condition, token_count)
            }
        };

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
                   current_owner as owner,
                   token.listing_currency_address as currency_address,
                   token.buy_in_progress
               FROM token
               WHERE token.contract_address = $1
                   AND token.chain_id = $2
                   AND ($3 = false OR (token.listing_start_amount IS NOT NULL AND token.listing_type != 'Auction'))
                   {} {}
               ORDER BY {}
               LIMIT $4 OFFSET $5",
            token_ids_condition, token_id_condition, order_by
        );

        let token_data_query_result: Vec<TokenDataDB> = sqlx::query_as(&tokens_data_query)
            .bind(contract_address)
            .bind(chain_id)
            .bind(buy_now)
            .bind(items_per_page)
            .bind((page - 1) * items_per_page)
            .fetch_all(self)
            .await?;

        let currencies = self.get_currencies().await?;
        let currencies_map: HashMap<String, Currency> = currencies
            .into_iter()
            .filter_map(|c| c.contract.clone().map(|contract| (contract, c)))
            .collect();

        let tokens_data: Vec<TokenData> = token_data_query_result
            .into_iter()
            .map(|token_data| TokenData {
                collection_address: token_data.collection_address,
                token_id: token_data.token_id,
                last_price: token_data.last_price,
                floor_difference: token_data.floor_difference,
                listed_at: token_data.listed_at,
                is_listed: token_data.is_listed,
                listing: token_data.listing_type.as_ref().and_then(|listing_type| {
                    token_data
                        .currency_address
                        .as_ref()
                        .and_then(|currency_address| {
                            currencies_map
                                .get(currency_address)
                                .map(|currency| TokenDataListing {
                                    is_auction: Some(listing_type == LISTING_TYPE_AUCTION_STR),
                                    currency: currency.clone(),
                                })
                        })
                }),
                price: token_data.price,
                metadata: token_data.metadata,
                owner: token_data.owner,
                buy_in_progress: token_data.buy_in_progress,
            })
            .collect();

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
                (token.listing_start_amount IS NOT NULL AND token.listing_type != 'Auction')
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
                c.contract_name as collection_name,
                token.listing_currency_address as currency_address
            FROM token
            INNER JOIN contract as c ON c.contract_address = token.contract_address
                AND c.chain_id = token.chain_id
            WHERE token.current_owner = $3
            AND (
                $4 = false OR
                (token.listing_start_amount IS NOT NULL AND token.listing_type != 'Auction')
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
                contract.contract_address as collection_address,
                te.currency_address,
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

        let token_activity_data_db: Vec<TokenActivityDataDB> = sqlx::query_as(&activity_sql_query)
            .bind(contract_address)
            .bind(chain_id)
            .bind(token_id)
            .fetch_all(self)
            .await?;

        let currencies: Vec<Currency> = sqlx::query_as!(
            Currency,
            r#"SELECT currency_address as contract, symbol, decimals FROM public.currency_mapping"#
        )
        .fetch_all(self)
        .await?;

        let token_activity_data: Vec<TokenActivityData> = token_activity_data_db
            .into_iter()
            .map(|sale| {
                let currency = currencies
                    .iter()
                    .find(|c| c.contract == sale.currency_address)
                    .cloned()
                    .unwrap_or_default();

                TokenActivityData {
                    time_stamp: sale.time_stamp,
                    transaction_hash: sale.transaction_hash,
                    activity_type: sale.activity_type,
                    metadata: sale.metadata,
                    collection_name: sale.collection_name,
                    collection_is_verified: sale.collection_is_verified,
                    collection_address: sale.collection_address,
                    price: sale.price,
                    from: sale.from,
                    to: sale.to,
                    currency,
                }
            })
            .collect();

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((token_activity_data, has_next_page, count))
    }
}

use crate::models::default::Currency;
use crate::models::portfolio::{OfferData, StatsData};
use crate::models::token::{
    TokenEventType, TokenPortfolioActivityData, TokenPortfolioActivityDataDB,
};
use crate::types::offer_type::OfferType;
use std::time::SystemTime;

use crate::utils::db_utils::event_type_list;
use async_trait::async_trait;
use sqlx::Error;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Row;

#[derive(FromRow)]
struct Count {
    total: i64,
}

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait DatabaseAccess: Send + Sync {
    async fn get_activity_data(
        &self,
        chain_id: &str,
        user_address: &str,
        page: i64,
        items_per_page: i64,
        direction: &str,
        types: &Option<Vec<TokenEventType>>,
    ) -> Result<(Vec<TokenPortfolioActivityData>, bool, i64), Error>;

    async fn get_offers_data(
        &self,
        chain_id: &str,
        user_address: &str,
        page: i64,
        items_per_page: i64,
        type_offer: OfferType,
    ) -> Result<(Vec<OfferData>, bool, i64), Error>;

    async fn get_stats_data(&self, chain_id: &str, user_address: &str) -> Result<StatsData, Error>;
}

#[async_trait]
impl DatabaseAccess for PgPool {
    async fn get_activity_data(
        &self,
        chain_id: &str,
        user_address: &str,
        page: i64,
        items_per_page: i64,
        direction: &str,
        types: &Option<Vec<TokenEventType>>,
    ) -> Result<(Vec<TokenPortfolioActivityData>, bool, i64), Error> {
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
                WHERE te.chain_id = $1
                    AND (te.from_address = $2 or te.to_address = $2)
                    {}
            ",
            types_filter
        );

        let count_sql_query = format!(
            "
            SELECT COUNT(*) AS total
            {}
            ",
            common_sql_query
        );

        let total_count: Count = sqlx::query_as(&count_sql_query)
            .bind(chain_id)
            .bind(user_address)
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

        let from_sql_query = format!(
            "
                FROM token_event te
                LEFT JOIN token_offer ON te.order_hash = token_offer.order_hash
                LEFT JOIN token ON te.token_id = token.token_id and te.contract_address = token.contract_address and te.chain_id = token.chain_id
                LEFT JOIN contract ON te.contract_address = contract.contract_address and te.chain_id = contract.chain_id
                WHERE te.chain_id = $1
                    AND (te.from_address = $2 or te.to_address = $2)
                    {}
            ",
            types_filter
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
                te.contract_address as collection_address,
                te.currency_address,
                token.metadata,
                contract.contract_name as collection_name,
                contract.is_verified as collection_is_verified,
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
            from_sql_query,
            direction,
            items_per_page,
            offset,
        );

        let token_activity_data_db: Vec<TokenPortfolioActivityDataDB> =
            sqlx::query_as(&activity_sql_query)
                .bind(chain_id)
                .bind(user_address)
                .fetch_all(self)
                .await?;

        let currencies: Vec<Currency> = sqlx::query_as!(
            Currency,
            r#"SELECT currency_address as contract, symbol, decimals FROM public.currency_mapping"#
        )
        .fetch_all(self)
        .await?;

        let token_activity_data: Vec<TokenPortfolioActivityData> = token_activity_data_db
            .into_iter()
            .map(|sale| {
                let currency = currencies
                    .iter()
                    .find(|c| c.contract == sale.currency_address)
                    .cloned();
                TokenPortfolioActivityData {
                    collection_name: sale.collection_name,
                    collection_address: sale.collection_address,
                    collection_is_verified: sale.collection_is_verified,
                    activity_type: sale.activity_type,
                    price: sale.price,
                    from: sale.from,
                    to: sale.to,
                    time_stamp: sale.time_stamp,
                    transaction_hash: sale.transaction_hash,
                    token_id: sale.token_id,
                    metadata: sale.metadata,
                    currency,
                }
            })
            .collect();

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((token_activity_data, has_next_page, count))
    }

    async fn get_offers_data(
        &self,
        chain_id: &str,
        user_address: &str,
        page: i64,
        items_per_page: i64,
        type_offer: OfferType,
    ) -> Result<(Vec<OfferData>, bool, i64), Error> {
        // FIXME: pagination assume that all offers used the same currency
        let offset = (page - 1) * items_per_page;
        let current_time: i64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => d.as_secs().try_into().unwrap(),
            Err(_) => 0,
        };

        let type_offer_query = type_offer.to_sql_condition();

        // common where_clause
        let where_clause = format!(
            "token_offer.chain_id = $1
             AND {}
             AND token_offer.status = 'PLACED'
             AND end_date > $3",
            type_offer_query
        );

        let total_count_query = format!(
            "SELECT COUNT(*) as count
            FROM token_offer
            WHERE {}",
            where_clause
        );

        let total_count = sqlx::query(&total_count_query)
            .bind(chain_id)
            .bind(user_address)
            .bind(current_time)
            .fetch_one(self)
            .await?;

        let count: i64 = total_count.get::<i64, _>("count");

        let token_offers_query = format!(
            "SELECT
                token_offer_id AS offer_id,
                hex_to_decimal(offer_amount) AS amount,
                end_date AS expire_at,
                order_hash as hash,
                token_offer.currency_address,
                to_address,
                offer_maker as source,
                token.token_id,
                contract.floor_price as collection_floor_price,
                contract.contract_address as collection_address,
                contract.contract_name as collection_name,
                contract.is_verified as is_verified,
                token.metadata,
                (token.listing_start_amount IS NOT NULL) as is_listed,
                (token.listing_type = 'Auction') as is_auction,
                token.listing_orderhash as listing_order_hash,
                token.listing_start_amount as listing_start_amount,
                token.listing_end_amount as listing_end_amount,
                token.listing_start_date as listing_start_date,
                token.listing_end_date as listing_end_date,
                cm.currency_address as currency_contract,
                cm.symbol as currency_symbol,
                cm.decimals as currency_decimals
            FROM token_offer
            LEFT JOIN contract ON token_offer.contract_address = contract.contract_address AND token_offer.chain_id = contract.chain_id
            LEFT JOIN token ON token_offer.contract_address = token.contract_address AND token_offer.chain_id = token.chain_id and token_offer.token_id = token.token_id
            LEFT JOIN currency_mapping cm on cm.currency_address = token.listing_currency_address and cm.chain_id = token.chain_id
            WHERE {}
            ORDER BY amount DESC, expire_at ASC
            LIMIT $4 OFFSET $5",
            where_clause
        );

        let token_offers_data = sqlx::query_as::<_, OfferData>(&token_offers_query)
            .bind(chain_id)
            .bind(user_address)
            .bind(current_time)
            .bind(items_per_page)
            .bind(offset)
            .fetch_all(self)
            .await?;

        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((token_offers_data, has_next_page, count))
    }

    async fn get_stats_data(&self, chain_id: &str, user_address: &str) -> Result<StatsData, Error> {
        let query = r#"
            SELECT SUM(contract.floor_price) AS total_value
            FROM token
            JOIN contract ON token.contract_address = contract.contract_address
            WHERE token.chain_id = $1
              AND token.current_owner = $2
        "#;

        let result = sqlx::query_as::<_, StatsData>(query)
            .bind(chain_id)
            .bind(user_address)
            .fetch_one(self)
            .await?;

        Ok(result)
    }
}

use crate::models::token::{TokenActivityData, TokenEventType};

use crate::utils::db_utils::event_type_list;
use async_trait::async_trait;
use sqlx::Error;
use sqlx::FromRow;
use sqlx::PgPool;

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
    ) -> Result<(Vec<TokenActivityData>, bool, i64), Error>;
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
    ) -> Result<(Vec<TokenActivityData>, bool, i64), Error> {
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

        let from_sql_query = format!(
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
                te.contract_address,
                token.metadata,
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
        let token_activity_data: Vec<TokenActivityData> = sqlx::query_as(&activity_sql_query)
            .bind(chain_id)
            .bind(user_address)
            .fetch_all(self)
            .await?;

        // Calculate if there is another page
        let total_pages = (count + items_per_page - 1) / items_per_page;
        let has_next_page = page < total_pages;

        Ok((token_activity_data, has_next_page, count))
    }
}

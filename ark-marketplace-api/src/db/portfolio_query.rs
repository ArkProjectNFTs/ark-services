use crate::db::portfolio_db_access;
use crate::models::portfolio::{OfferData, StatsData};
use crate::models::token::{TokenEventType, TokenPortfolioActivityData};
use crate::types::offer_type::OfferType;

#[allow(clippy::too_many_arguments)]
pub async fn get_activity_data<D: portfolio_db_access::DatabaseAccess + Sync>(
    db_access: &D,
    chain_id: &str,
    user_address: &str,
    page: i64,
    items_per_page: i64,
    direction: &str,
    types: &Option<Vec<TokenEventType>>,
) -> Result<(Vec<TokenPortfolioActivityData>, bool, i64), sqlx::Error> {
    db_access
        .get_activity_data(
            chain_id,
            user_address,
            page,
            items_per_page,
            direction,
            types,
        )
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_offers_data<D: portfolio_db_access::DatabaseAccess + Sync>(
    db_access: &D,
    chain_id: &str,
    user_address: &str,
    page: i64,
    items_per_page: i64,
    type_offer: OfferType,
) -> Result<(Vec<OfferData>, bool, i64), sqlx::Error> {
    db_access
        .get_offers_data(chain_id, user_address, page, items_per_page, type_offer)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_stats_data<D: portfolio_db_access::DatabaseAccess + Sync>(
    db_access: &D,
    chain_id: &str,
    user_address: &str,
) -> Result<StatsData, sqlx::Error> {
    db_access.get_stats_data(chain_id, user_address).await
}

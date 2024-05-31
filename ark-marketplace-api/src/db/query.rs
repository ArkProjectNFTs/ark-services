use crate::db::db_access::DatabaseAccess;
use crate::models::collection::CollectionData;
use crate::models::token::{TokenData, TokenPortfolioData};

pub async fn get_collections_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    page: i64,
    items_per_page: i64,
    time_range: &str,
) -> Result<Vec<CollectionData>, sqlx::Error> {
    db_access
        .get_collections_data(page, items_per_page, time_range)
        .await
}

pub async fn get_collection_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    chain_id: &str,
) -> Result<CollectionData, sqlx::Error> {
    db_access
        .get_collection_data(contract_address, chain_id)
        .await
}

pub async fn get_tokens_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
    page: i64,
    items_per_page: i64,
    buy_now: bool,
    sort: &str,
    direction: &str,
) -> Result<(Vec<TokenData>, bool), sqlx::Error> {
    db_access
        .get_tokens_data(
            contract_address,
            page,
            items_per_page,
            buy_now,
            sort,
            direction,
        )
        .await
}

pub async fn get_tokens_portfolio_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    user_address: &str,
    page: i64,
    items_per_page: i64,
    buy_now: bool,
    sort: &str,
    direction: &str,
    collection: &str,
) -> Result<(Vec<TokenPortfolioData>, bool), sqlx::Error> {
    db_access
        .get_tokens_portfolio_data(
            user_address,
            page,
            items_per_page,
            buy_now,
            sort,
            direction,
            collection,
        )
        .await
}

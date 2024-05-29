use crate::db::db_access::DatabaseAccess;
use crate::models::collection::CollectionData;
use crate::models::token::TokenData;

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
) -> Result<CollectionData, sqlx::Error> {
    db_access.get_collection_data(contract_address).await
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

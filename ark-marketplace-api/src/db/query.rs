use crate::db::db_access::DatabaseAccess;
use crate::models::collection::CollectionData;
use crate::models::token::TokenData;
use std::collections::HashMap;

pub async fn get_collection_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    page: i64,
    items_per_page: i64,
    time_range: &str,
) -> Result<Vec<CollectionData>, sqlx::Error> {
    db_access.get_collection_data(page, items_per_page, time_range).await
}

pub async fn get_tokens_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    page: i64,
    items_per_page: i64,
) -> Result<Vec<TokenData>, sqlx::Error> {
    db_access.get_tokens_data(page, items_per_page).await
}

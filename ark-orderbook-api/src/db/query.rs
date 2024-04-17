use crate::db::db_access::DatabaseAccess;
use crate::models::token::{TokenData, TokenWithHistory, TokenWithOffers};

pub async fn get_token_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    token_address: &str,
    token_id: &str,
) -> Result<TokenData, sqlx::Error> {
    db_access.get_token_data(token_address, token_id).await
}

pub async fn get_token_by_collection_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    token_address: &str,
) -> Result<Vec<TokenData>, sqlx::Error> {
    db_access.get_token_by_collection_data(token_address).await
}

pub async fn get_token_history_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    token_address: &str,
    token_id: &str,
) -> Result<TokenWithHistory, sqlx::Error> {
    db_access
        .get_token_history_data(token_address, token_id)
        .await
}

pub async fn get_token_offers_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    token_address: &str,
    token_id: &str,
) -> Result<TokenWithOffers, sqlx::Error> {
    db_access
        .get_token_offers_data(token_address, token_id)
        .await
}

pub async fn get_tokens_by_account_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    owner: &str,
) -> Result<Vec<TokenData>, sqlx::Error> {
    db_access.get_tokens_by_owner_data(owner).await
}

pub async fn delete_token_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    token_address: &str,
    token_id: &str,
) -> Result<u64, sqlx::Error> {
    db_access.delete_token_data(token_address, token_id).await
}

pub async fn flush_all_data_query<D: DatabaseAccess + Sync>(
    db_access: &D,
) -> Result<u64, sqlx::Error> {
    db_access.flush_all_data().await
}

pub async fn delete_migrations_query<D: DatabaseAccess + Sync>(
    db_access: &D,
) -> Result<u64, sqlx::Error> {
    db_access.delete_migrations().await
}

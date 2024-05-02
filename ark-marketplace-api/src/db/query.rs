use crate::db::db_access::DatabaseAccess;
use crate::models::token::{TokenData, TokenWithHistory, TokenWithOffers};

pub async fn get_token_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    token_address: &str,
    token_id: &str,
) -> Result<TokenData, sqlx::Error> {
    db_access.get_token_data(token_address, token_id).await
}


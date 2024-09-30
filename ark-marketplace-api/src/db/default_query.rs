use crate::db::default_db_access;
use crate::models::default::{LastSale, LiveAuction, Trending};

#[allow(clippy::too_many_arguments)]
pub async fn get_last_sales<D: default_db_access::DatabaseAccess + Sync>(
    db_access: &D,
) -> Result<Vec<LastSale>, sqlx::Error> {
    db_access.get_last_sales().await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_live_auctions<D: default_db_access::DatabaseAccess + Sync>(
    db_access: &D,
) -> Result<Vec<LiveAuction>, sqlx::Error> {
    db_access.get_live_auctions().await
}

#[allow(clippy::too_many_arguments)]
pub async fn get_trending<D: default_db_access::DatabaseAccess + Sync>(
    db_access: &D,
    time_range: &str,
) -> Result<Vec<Trending>, sqlx::Error> {
    db_access.get_trending(time_range).await
}

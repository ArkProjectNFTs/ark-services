use crate::db::default_db_access;
use crate::models::default::LastSale;

#[allow(clippy::too_many_arguments)]
pub async fn get_last_sales<D: default_db_access::DatabaseAccess + Sync>(
    db_access: &D,
) -> Result<Vec<LastSale>, sqlx::Error> {
    db_access.get_last_sales().await
}

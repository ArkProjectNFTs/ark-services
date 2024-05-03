use crate::db::db_access::DatabaseAccess;
use crate::models::collection::CollectionData;

pub async fn get_collection_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    page: i64,
    items_per_page: i64,
) -> Result<Vec<CollectionData>, sqlx::Error> {
    db_access.get_collection_data(page, items_per_page).await
}

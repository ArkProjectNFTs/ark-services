use crate::db::db_access::DatabaseAccess;
use crate::models::collection::CollectionData;

pub async fn get_collection_data<D: DatabaseAccess + Sync>(
    db_access: &D,
    contract_address: &str,
) -> Result<CollectionData, sqlx::Error> {
    db_access.get_collection_data(contract_address).await
}

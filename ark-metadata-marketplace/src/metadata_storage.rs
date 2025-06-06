use arkproject::{
    metadata::{
        storage::Storage,
        types::{StorageError, TokenMetadata, TokenWithoutMetadata},
    },
    sana,
};
use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;
use tracing::{error, trace};

pub struct MetadataSqlStorage {
    pool: PgPool,
}

impl MetadataSqlStorage {
    pub async fn new_pg(db_url: &str) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(db_url)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for MetadataSqlStorage {
    async fn update_all_token_metadata_status(
        &self,
        contract_address: &str,
        chain_id: &str,
        metadata_status: &str,
    ) -> Result<(), StorageError> {
        trace!(
            "Updating token metadata status. Contract address: {} - - Chain ID: {} - Status: {}",
            contract_address,
            chain_id,
            metadata_status
        );

        let res = sqlx::query(
            "UPDATE token SET updated_timestamp=EXTRACT(epoch FROM now())::bigint, metadata_status = $1
            WHERE contract_address = $2 AND chain_id = $3",
        )
        .bind(metadata_status)
        .bind(contract_address)
        .bind(chain_id)
        .execute(&self.pool)
        .await;

        if res.is_err() {
            error!("Failed to update token metadata status. Error: {:?}", res);
            return Err(StorageError::DatabaseError(res.unwrap_err().to_string()));
        }

        Ok(())
    }

    async fn set_contract_refreshing_status(
        &self,
        contract_address: &str,
        chain_id: &str,
        is_refreshing: bool,
    ) -> Result<(), StorageError> {
        let query = "
            UPDATE contract 
            SET is_refreshing = $1, updated_timestamp = EXTRACT(epoch FROM now())::bigint 
            WHERE contract_address = $2 AND chain_id = $3";

        let res = sqlx::query(query)
            .bind(is_refreshing)
            .bind(contract_address)
            .bind(chain_id)
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to update contract is refreshing. Error: {:?}", e);
                Err(StorageError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn update_token_metadata_status(
        &self,
        contract_address: &str,
        token_id: &str,
        chain_id: &str,
        metadata_status: &str,
    ) -> Result<(), StorageError> {
        trace!("Updating token metadata status. Contract address: {} - Token ID: {} - Chain ID: {} - Status: {}", contract_address, token_id, chain_id, metadata_status);

        let res = sqlx::query(
            "UPDATE token SET updated_timestamp=EXTRACT(epoch FROM now())::bigint, metadata_status = $1
            WHERE contract_address = $2 AND chain_id = $3 AND token_id = $4",
        )
        .bind(metadata_status)
        .bind(contract_address)
        .bind(chain_id)
        .bind(token_id)
        .execute(&self.pool)
        .await;

        if res.is_err() {
            error!("Failed to update token metadata status. Error: {:?}", res);
            return Err(StorageError::DatabaseError(res.unwrap_err().to_string()));
        }

        Ok(())
    }

    async fn register_token_metadata(
        &self,
        contract_address: &str,
        token_id: &str,
        chain_id: &str,
        token_metadata: TokenMetadata,
    ) -> Result<(), StorageError> {
        let query = "
        UPDATE token
        SET updated_timestamp = EXTRACT(epoch FROM now())::bigint, metadata = $4::jsonb, raw_metadata = $5, metadata_status = $6, metadata_updated_at = $7
        WHERE contract_address = $1 AND chain_id = $2 AND token_id = $3";

        let normalized_metadata_json =
            serde_json::to_string(&token_metadata.normalized).map_err(|e| {
                error!("Failed to serialize token metadata. Error: {}", e);
                StorageError::DatabaseError(e.to_string())
            })?;

        match sqlx::query(query)
            .bind(contract_address)
            .bind(chain_id)
            .bind(token_id)
            .bind(normalized_metadata_json)
            .bind(token_metadata.raw)
            .bind("OK".to_string())
            .bind(token_metadata.metadata_updated_at)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to register token metadata. Error: {}", e);
                Err(StorageError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn find_tokens_without_metadata(
        &self,
        filter: Option<(String, String)>,
        refresh_collection: bool,
    ) -> Result<Vec<TokenWithoutMetadata>, StorageError> {
        let base_query = "SELECT t.contract_address, t.token_id, t.chain_id, c.is_verified, c.save_images FROM token t INNER JOIN contract c on c.contract_address = t.contract_address and c.chain_id = t.chain_id  WHERE c.is_spam = false AND c.is_nsfw  = false AND c.contract_type = 'ERC721'";
        let status = if refresh_collection {
            "COLLECTION_TO_REFRESH"
        } else {
            "TO_REFRESH"
        };

        let (query, params) = match filter {
            Some((contract_address, chain_id)) => (
                format!(
                    "{} AND t.metadata_status = '{}' AND t.chain_id = $1 AND t.contract_address = $2 LIMIT 100",
                    base_query, status
                ),
                vec![chain_id, contract_address],
            ),
            None => (
                format!(
                    "{} AND t.metadata_status = '{}' LIMIT 100",
                    base_query, status
                ),
                vec![],
            ),
        };

        let mut query_builder = sqlx::query(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = match query_builder.fetch_all(&self.pool).await {
            Ok(rows) => rows,
            Err(e) => {
                error!("Failed to fetch token ids without metadata. Error: {}", e);
                return Err(StorageError::DatabaseError(e.to_string()));
            }
        };

        // Process the rows
        if rows.is_empty() {
            return Ok(vec![]);
        }

        let tokens: Vec<TokenWithoutMetadata> = rows
            .into_iter()
            .filter_map(|row| {
                sana::storage::sqlx::types::TokenWithoutMetadata::from_row(&row)
                    .ok()
                    .map(|res| TokenWithoutMetadata {
                        contract_address: res.contract_address,
                        token_id: res.token_id,
                        chain_id: res.chain_id,
                        is_verified: res.is_verified,
                        save_images: res.save_images,
                    })
            })
            .collect();

        Ok(tokens)
    }
}

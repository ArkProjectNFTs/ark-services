use arkproject::{
    metadata::{
        storage::Storage,
        types::{StorageError, TokenMetadata, TokenWithoutMetadata},
    },
    sana,
};
use async_trait::async_trait;
use sqlx::{any::AnyPoolOptions, AnyPool, FromRow};
use tracing::{error, info};

pub struct MetadataSqlStorage {
    pool: AnyPool,
}

impl MetadataSqlStorage {
    pub async fn new_any(db_url: &str) -> Result<Self, StorageError> {
        sqlx::any::install_default_drivers();

        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect(db_url)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for MetadataSqlStorage {
    async fn update_token_metadata_status(
        &self,
        contract_address: &str,
        token_id: &str,
        chain_id: &str,
        metadata_status: &str,
    ) -> Result<(), StorageError> {
        info!("Updating token metadata status. Contract address: {} - Token ID: {} - Chain ID: {} - Status: {}", contract_address, token_id, chain_id, metadata_status);

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

    async fn find_token_ids_without_metadata(
        &self,
        filter: Option<(String, String)>,
    ) -> Result<Vec<TokenWithoutMetadata>, StorageError> {
        let base_query = "SELECT t.contract_address, t.chain_id, t.token_id, c.is_verified::text FROM token t INNER JOIN contract c on c.contract_address = t.contract_address and c.chain_id = t.chain_id  WHERE metadata_status = 'TO_REFRESH'";
        let (query, params) = if let Some((chain_id, contract_address)) = filter {
            (
                format!(
                    "{} AND t.chain_id = $1 AND t.contract_address = $2 LIMIT 100",
                    base_query
                ),
                vec![chain_id, contract_address],
            )
        } else {
            (format!("{} LIMIT 100", base_query), vec![])
        };

        let mut query_builder = sqlx::query(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        match query_builder.fetch_all(&self.pool).await {
            Ok(rows) => {
                if rows.is_empty() {
                    return Ok(vec![]);
                } else {
                    let mut tokens: Vec<TokenWithoutMetadata> = vec![];
                    for row in rows {
                        if let Ok(res) =
                            sana::storage::sqlx::types::TokenWithoutMetadata::from_row(&row)
                        {
                            let is_verified = match res.is_verified.as_str() {
                                "true" => true,
                                "false" => false,
                                _ => false,
                            };

                            tokens.push(TokenWithoutMetadata {
                                contract_address: res.contract_address,
                                token_id: res.token_id,
                                chain_id: res.chain_id,
                                is_verified,
                            });
                        }
                    }
                    return Ok(tokens);
                }
            }
            Err(e) => {
                error!("Failed to fetch token ids without metadata. Error: {}", e);
                Err(StorageError::DatabaseError(e.to_string()))
            }
        }
    }
}

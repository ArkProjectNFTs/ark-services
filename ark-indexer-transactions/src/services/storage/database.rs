use chrono::Utc;

use crate::interfaces::contract::ContractType;
use crate::interfaces::contract::{NFTInfo, TransactionInfo};
use crate::interfaces::event::{ERCCompliance, ErcAction, EventType};
use sqlx::PgPool;

use super::Storage;

#[derive(Clone)]
pub struct DatabaseStorage {
    pool: PgPool,
}

impl DatabaseStorage {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPool::connect(database_url).await?;
        Ok(DatabaseStorage { pool })
    }
}

impl DatabaseStorage {
    // async fn get_nft_info(
    //     &self,
    //     contract_address: &str,
    //     token_id: String,
    // ) -> Result<Option<NFTInfo>, Box<dyn std::error::Error>> {
    //     let query = r#"
    //         SELECT contract_address, token_id, name, symbol, metadata_uri, owner, chain_id, block_hash, indexed_at
    //         FROM nft_info
    //         WHERE contract_address = $1 AND token_id = $2
    //     "#;

    //     let nft_info = sqlx::query_as::<_, NFTInfo>(query)
    //         .bind(contract_address)
    //         .bind(token_id)
    //         .fetch_optional(&self.pool)
    //         .await?;

    //     Ok(nft_info)
    // }

    // async fn get_transaction_info(&self, tx_hash: &str) -> Result<Option<TransactionInfo>, Box<dyn std::error::Error>> {
    //     let query = r#"
    //         SELECT tx_hash, from_address, to_address, value, timestamp, token_id, contract_address, contract_type, block_hash, indexed_at
    //         FROM transaction_info
    //         WHERE tx_hash = $1
    //     "#;

    //     let tx_info = sqlx::query_as::<_, TransactionInfo>(query)
    //         .bind(tx_hash)
    //         .fetch_optional(&self.pool)
    //         .await?;

    //     Ok(tx_info)
    // }

    // async fn is_event_already_indexed(
    //     &self,
    //     event_id: &str,
    //     block_hash: &str,
    // ) -> Result<bool, Box<dyn std::error::Error>> {
    //     let query = r#"
    //         SELECT COUNT(*)
    //         FROM transaction_info
    //         WHERE tx_hash = $1 AND block_hash = $2
    //     "#;

    //     let count: (i64,) = sqlx::query_as(query)
    //         .bind(event_id)
    //         .bind(block_hash)
    //         .fetch_one(&self.pool)
    //         .await?;

    //     Ok(count.0 > 0)
    // }

    // async fn mark_block_indexed(&self, block_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     let query = r#"
    //         INSERT INTO indexed_blocks (block_hash, indexed_at)
    //         VALUES ($1, $2)
    //         ON CONFLICT (block_hash) DO NOTHING
    //     "#;

    //     sqlx::query(query)
    //         .bind(block_hash)
    //         .bind(Utc::now())
    //         .execute(&self.pool)
    //         .await?;

    //     Ok(())
    // }
}

#[async_trait::async_trait]
impl Storage for DatabaseStorage {
    async fn store_transaction_info(
        &self,
        tx_info: TransactionInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_id = format!("{}_{}", tx_info.tx_hash, tx_info.event_id);
        // println!("tx_info: {:?}", tx_info);
        sqlx::query_as!(
                TransactionInfoModel,
                r#"
                INSERT INTO transaction_info (
                    tx_hash, event_id, from_address, to_address, value, timestamp, token_id, contract_address, contract_type, block_hash, event_type, erc_compliance, erc_action, indexed_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
                )
                ON CONFLICT (tx_hash, event_id) DO UPDATE
                SET from_address = EXCLUDED.from_address, 
                    to_address = EXCLUDED.to_address, 
                    value = EXCLUDED.value,
                    timestamp = EXCLUDED.timestamp, 
                    token_id = EXCLUDED.token_id, 
                    contract_address = EXCLUDED.contract_address,
                    contract_type = EXCLUDED.contract_type, 
                    block_hash = EXCLUDED.block_hash, 
                    indexed_at = EXCLUDED.indexed_at
                "#,
                &tx_info.tx_hash,
                &event_id,
                &tx_info.from,
                &tx_info.to,
                tx_info.value,
                tx_info.timestamp as i64,
                tx_info.token_id,
                &tx_info.contract_address,
                tx_info.contract_type as ContractType,  // Ensure the contract type is passed as a string
                &tx_info.block_hash,
                tx_info.event_type as EventType,  // Ensure event type is passed as a string
                tx_info.compliance as ERCCompliance,  // Ensure compliance is passed as a string
                tx_info.action as ErcAction,  // Ensure action is passed as a string
                Utc::now()
            )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn store_nft_info(
        &self,
        nft_info: NFTInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            INSERT INTO nft_info (
                contract_address, token_id, name, symbol, metadata_uri, owner, chain_id, block_hash, indexed_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (contract_address, token_id) DO UPDATE
            SET name = EXCLUDED.name, symbol = EXCLUDED.symbol, metadata_uri = EXCLUDED.metadata_uri,
                owner = EXCLUDED.owner, chain_id = EXCLUDED.chain_id, block_hash = EXCLUDED.block_hash, indexed_at = EXCLUDED.indexed_at
        "#;

        sqlx::query(query)
            .bind(&nft_info.contract_address)
            .bind(&nft_info.token_id)
            .bind(&nft_info.name)
            .bind(&nft_info.symbol)
            .bind(&nft_info.metadata_uri)
            .bind(&nft_info.owner)
            .bind(&nft_info.chain_id)
            .bind(&nft_info.block_hash)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

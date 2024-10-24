use chrono::Utc;

use crate::interfaces::contract::ContractType;
use crate::interfaces::contract::{ContractInfo, NFTInfo, TransactionInfo};
use crate::interfaces::event::{ERCCompliance, ErcAction, EventType};
use bigdecimal::BigDecimal;
use num_bigint::BigUint;
use sqlx::PgPool;
use std::str::FromStr;

use super::{NFTInfoStorage, Storage, TransactionInfoStorage};

trait BigDecimalHex {
    fn to_hex_string(&self) -> String;
}

impl BigDecimalHex for BigDecimal {
    fn to_hex_string(&self) -> String {
        // Convertir BigDecimal en string sans notation scientifique
        let decimal_str = self.to_string().replace(".0", "");

        // Convertir en BigUint
        if let Ok(big_uint) = BigUint::from_str(&decimal_str) {
            // Convertir en bytes pour la représentation hex
            let bytes = big_uint.to_bytes_be();

            // Convertir en hex string avec padding à 64 caractères
            let hex = hex::encode(bytes);
            format!("0x{:0>64}", hex)
        } else {
            format!("0x{:0>64}", "0")
        }
    }
}
#[derive(Clone)]
pub struct DatabaseStorage {
    pool: PgPool,
}

impl DatabaseStorage {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPool::connect(database_url).await?;
        Ok(DatabaseStorage { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
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
impl TransactionInfoStorage for DatabaseStorage {
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
                    tx_hash, event_id, from_address, to_address, value, timestamp, token_id, contract_address, contract_type, block_hash, event_type, erc_compliance, erc_action, indexed_at, sub_event_id
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
                )
                ON CONFLICT (tx_hash, event_id, sub_event_id) DO UPDATE
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
                Utc::now(),
                tx_info.sub_event_id,
            )
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl NFTInfoStorage for DatabaseStorage {
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

    async fn store_token(
        &self,
        nft_info: NFTInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            INSERT INTO token (
                contract_address, chain_id, token_id, token_id_hex, metadata_status, current_owner, block_timestamp, updated_timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (contract_address, chain_id, token_id) DO UPDATE
            SET current_owner = EXCLUDED.current_owner, chain_id = EXCLUDED.chain_id, block_timestamp = EXCLUDED.block_timestamp, updated_timestamp = EXCLUDED.updated_timestamp
        "#;
        let mut token_id_hex: Option<String> = None;
        if let Some(token_id) = nft_info.token_id.clone() {
            token_id_hex = Some(token_id.to_hex_string())
        } else {
            println!("Invalid Token with info: {:?}", nft_info);
        }

        sqlx::query(query)
            .bind(&nft_info.contract_address)
            .bind(&nft_info.chain_id)
            .bind(&nft_info.token_id)
            .bind(token_id_hex)
            .bind("TO_REFRESH")
            .bind(&nft_info.owner)
            .bind(nft_info.block_timestamp as i64)
            .bind(Utc::now().timestamp())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn store_contract(
        &self,
        contract_info: ContractInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            INSERT INTO contract (
                contract_address, chain_id, contract_type, contract_name, contract_symbol, updated_timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (contract_address, chain_id) DO UPDATE
            SET contract_type = EXCLUDED.contract_type, contract_name = EXCLUDED.contract_name, contract_symbol = EXCLUDED.contract_symbol, chain_id = EXCLUDED.chain_id, updated_timestamp = EXCLUDED.updated_timestamp
        "#;

        sqlx::query(query)
            .bind(&contract_info.contract_address)
            .bind(&contract_info.chain_id)
            .bind(&contract_info.contract_type)
            .bind(&contract_info.name)
            .bind(&contract_info.symbol)
            .bind(Utc::now().timestamp())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn store_token_event(
        &self,
        tx_info: TransactionInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            INSERT INTO token_event (
                token_event_id, contract_address, chain_id, token_id, token_id_hex, event_type, block_timestamp, transaction_hash, to_address, from_address, amount, token_sub_event_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (token_event_id, token_sub_event_id) DO UPDATE
            SET chain_id = EXCLUDED.chain_id, block_timestamp = EXCLUDED.block_timestamp, transaction_hash = EXCLUDED.transaction_hash
        "#;

        let mut token_id_hex: Option<String> = None;
        if let Some(token_id) = tx_info.token_id.clone() {
            token_id_hex = Some(token_id.to_hex_string())
        } else {
            println!("Invalid Token with info: {:?}", tx_info);
        }
        let event_id = format!("{}_{}", tx_info.tx_hash, tx_info.event_id);

        sqlx::query(query)
            .bind(event_id)
            .bind(&tx_info.contract_address)
            .bind(&tx_info.chain_id)
            .bind(tx_info.token_id)
            .bind(token_id_hex)
            .bind(&tx_info.event_type)
            .bind(tx_info.timestamp as i64)
            .bind(&tx_info.tx_hash)
            .bind(&tx_info.to)
            .bind(&tx_info.from)
            .bind(&tx_info.value)
            .bind(&tx_info.sub_event_id)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

impl Storage for DatabaseStorage {}

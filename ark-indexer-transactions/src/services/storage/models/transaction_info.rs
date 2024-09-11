use serde::{Deserialize, Serialize};

use crate::interfaces::{
    contract::ContractType,
    event::{ERCCompliance, ErcAction, EventType},
};

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TransactionInfoModel {
    pub id: i32,
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub value: Option<String>,
    pub timestamp: u64,
    pub token_id: Option<String>, // Pour ERC721 / ERC1155
    pub event_type: EventType,
    pub erc_compliance: ERCCompliance,
    pub erc_action: ErcAction,
    pub contract_address: String,
    pub contract_type: ContractType,
    pub block_hash: String,
    pub event_hash: String,
    #[serde(rename = "indexed_at")]
    pub indexed_at: Option<chrono::DateTime<chrono::Utc>>,
}

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull,
    postgres::{PgArgumentBuffer, PgValueRef},
    Decode, Encode, FromRow, Postgres,
};

use starknet::core::types::Felt;
// use starknet::core::types::String;
use sqlx::types::BigDecimal;
use starknet::providers::ProviderError;

use super::event::{ERCCompliance, ErcAction, EventType};

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct ContractInfo {
    pub chain_id: String,
    pub contract_address: String,
    pub contract_type: ContractType,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct NFTInfo {
    pub contract_address: String,
    pub token_id: Option<BigDecimal>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub metadata_uri: Option<String>,
    pub owner: String,
    pub chain_id: String,
    pub tx_hash: String,
    pub block_hash: String,
    pub block_timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenInfo {
    pub contract_address: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Option<String>,
    pub chain_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct TransactionInfo {
    pub tx_hash: String,
    pub event_id: u64,
    pub chain_id: String,
    pub from: String,
    pub to: String,
    pub value: Option<BigDecimal>,
    pub timestamp: u64,
    pub token_id: Option<BigDecimal>, // Pour ERC721 / ERC1155
    pub event_type: EventType,
    pub compliance: ERCCompliance,
    pub action: ErcAction,
    pub contract_address: String,
    pub contract_type: ContractType,
    pub block_hash: String,
    pub sub_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ERC20Event {
    Transfer {
        from: Felt,
        to: Felt,
        value: BigDecimal,
    },
    Approval {
        owner: Felt,
        spender: Felt,
        value: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ERC721Event {
    Transfer {
        from: Felt,
        to: Felt,
        token_id_low: Felt,
        token_id_high: Felt,
    },
    Approval {
        owner: Felt,
        approved: Felt,
        token_id_low: Felt,
        token_id_high: Felt,
    },
    ApprovalForAll {
        owner: Felt,
        operator: Felt,
        approved: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ERC1155Event {
    TransferSingle {
        operator: Felt,
        from: Felt,
        to: Felt,
        id_low: Felt,
        id_high: Felt,
        value: BigDecimal,
    },
    TransferBatch {
        operator: Felt,
        from: Felt,
        to: Felt,
        ids: Vec<(Felt, Felt)>,
        values: Vec<BigDecimal>,
    },
    ApprovalForAll {
        owner: Felt,
        operator: Felt,
        approved: bool,
    },
    URI {
        value: String,
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ERC1400Event {
    Transfer {
        from: Felt,
        to: Felt,
        value: BigDecimal,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    ERC20,
    ERC721,
    ERC1155,
    ERC1400,
    Other,
}

impl FromStr for ContractType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ERC20" => Ok(ContractType::ERC20),
            "ERC721" => Ok(ContractType::ERC721),
            "ERC1155" => Ok(ContractType::ERC1155),
            "ERC1400" => Ok(ContractType::ERC1400),
            _ => Ok(ContractType::Other),
        }
    }
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractType::Other => write!(f, "OTHER"),
            ContractType::ERC721 => write!(f, "ERC721"),
            ContractType::ERC1155 => write!(f, "ERC1155"),
            ContractType::ERC20 => write!(f, "ERC20"),
            ContractType::ERC1400 => write!(f, "ERC1400"),
        }
    }
}

impl AsRef<str> for ContractType {
    fn as_ref(&self) -> &str {
        match self {
            ContractType::Other => "OTHER",
            ContractType::ERC721 => "ERC721",
            ContractType::ERC1155 => "ERC1155",
            ContractType::ERC20 => "ERC20",
            ContractType::ERC1400 => "ERC1400",
        }
    }
}

impl sqlx::Type<Postgres> for ContractType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("contract_type") // This should match the enum type name in PostgreSQL
    }
}

impl Encode<'_, Postgres> for ContractType {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let str_val = self.as_ref();
        <&str as Encode<Postgres>>::encode(str_val, buf)
    }

    fn size_hint(&self) -> usize {
        self.to_string().len()
    }
}

impl<'r> Decode<'r, Postgres> for ContractType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        Ok(ContractType::from_str(s).map_err(|_| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to decode ContractType",
            ))
        })?)
    }
}

/// Generic errors for starknet client.
#[derive(Debug, thiserror::Error)]
pub enum StarknetClientError {
    #[error("A contract error occurred: {0}")]
    Contract(String),
    #[error("Entry point not found: {0}")]
    EntrypointNotFound(String),
    #[error("Input too long for arguments")]
    InputTooLong,
    #[error("Input too short for arguments")]
    InputTooShort,
    #[error("Invalid value conversion: {0}")]
    Conversion(String),
    #[error("Starknet-rs provider error: {0}")]
    Provider(ProviderError),
    #[error("Other error: {0}")]
    Other(String),
}

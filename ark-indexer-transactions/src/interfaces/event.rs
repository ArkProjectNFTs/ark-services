use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull,
    postgres::{PgArgumentBuffer, PgValueRef},
    Decode, Encode, Postgres,
};
use starknet::{core::types::Felt, macros::selector};

pub const TRANSFER: Felt = selector!("Transfer");
pub const APPROVAL: Felt = selector!("Approval");
pub const APPROVAL_FOR_ALL: Felt = selector!("ApprovalForAll");
pub const TRANSFER_SINGLE: Felt = selector!("TransferSingle");
pub const TRANSFER_BATCH: Felt = selector!("TransferBatch");
pub const URI: Felt = selector!("URI");
pub const TRANSFER_BY_PARTITION: Felt = selector!("TransferByPartition");
pub const CHANGED_PARTITION: Felt = selector!("ChangedPartition");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ERCCompliance {
    OPENZEPPELIN,
    OTHER,
}

impl FromStr for ERCCompliance {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OPENZEPPELIN" => Ok(ERCCompliance::OPENZEPPELIN),
            _ => Ok(ERCCompliance::OTHER),
        }
    }
}

impl std::fmt::Display for ERCCompliance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ERCCompliance::OTHER => write!(f, "OTHER"),
            ERCCompliance::OPENZEPPELIN => write!(f, "OPENZEPPELIN"),
        }
    }
}

impl AsRef<str> for ERCCompliance {
    fn as_ref(&self) -> &str {
        match self {
            ERCCompliance::OTHER => "OTHER",
            ERCCompliance::OPENZEPPELIN => "OPENZEPPELIN",
        }
    }
}

impl sqlx::Type<Postgres> for ERCCompliance {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("erc_compliance") // This should match the enum type name in PostgreSQL
    }
}

impl Encode<'_, Postgres> for ERCCompliance {
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

impl<'r> Decode<'r, Postgres> for ERCCompliance {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        ERCCompliance::from_str(s).map_err(|_| "Failed to decode ERCCompliance".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    Transfer,
    Approval,
    ApprovalForAll,
    TransferSingle,
    TransferBatch,
    URI,
    TransferByPartition,
    ChangedPartition,
    Other,
}

impl FromStr for EventType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Transfer" => Ok(EventType::Transfer),
            "Approval" => Ok(EventType::Approval),
            "ApprovalForAll" => Ok(EventType::ApprovalForAll),
            "TransferSingle" => Ok(EventType::TransferSingle),
            "TransferBatch" => Ok(EventType::TransferBatch),
            "URI" => Ok(EventType::URI),
            "TransferByPartition" => Ok(EventType::TransferByPartition),
            "ChangedPartition" => Ok(EventType::ChangedPartition),
            _ => Ok(EventType::Other),
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Transfer => write!(f, "Transfer"),
            EventType::Approval => write!(f, "Approval"),
            EventType::ApprovalForAll => write!(f, "ApprovalForAll"),
            EventType::TransferSingle => write!(f, "TransferSingle"),
            EventType::TransferBatch => write!(f, "TransferBatch"),
            EventType::URI => write!(f, "URI"),
            EventType::TransferByPartition => write!(f, "TransferByPartition"),
            EventType::ChangedPartition => write!(f, "ChangedPartition"),
            _ => write!(f, "Other"),
        }
    }
}

impl AsRef<str> for EventType {
    fn as_ref(&self) -> &str {
        match self {
            EventType::Transfer => "Transfer",
            EventType::Approval => "Approval",
            EventType::ApprovalForAll => "ApprovalForAll",
            EventType::TransferSingle => "TransferSingle",
            EventType::TransferBatch => "TransferBatch",
            EventType::URI => "URI",
            EventType::TransferByPartition => "TransferByPartition",
            EventType::ChangedPartition => "ChangedPartition",
            EventType::Other => "Other",
        }
    }
}

impl sqlx::Type<Postgres> for EventType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("event_type") // This should match the enum type name in PostgreSQL
    }
}

impl Encode<'_, Postgres> for EventType {
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

impl<'r> Decode<'r, Postgres> for EventType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        EventType::from_str(s).map_err(|_| "Failed to decode EventType".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErcAction {
    MINT,
    BURN,
    OTHER,
}

impl FromStr for ErcAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MINT" => Ok(ErcAction::MINT),
            "BURN" => Ok(ErcAction::BURN),
            _ => Ok(ErcAction::OTHER),
        }
    }
}

impl std::fmt::Display for ErcAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErcAction::OTHER => write!(f, "OTHER"),
            ErcAction::MINT => write!(f, "MINT"),
            ErcAction::BURN => write!(f, "BURN"),
        }
    }
}

impl sqlx::Type<Postgres> for ErcAction {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("erc_action") // This should match the enum type name in PostgreSQL
    }
}

impl Encode<'_, Postgres> for ErcAction {
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

impl AsRef<str> for ErcAction {
    fn as_ref(&self) -> &str {
        match self {
            ErcAction::OTHER => "OTHER",
            ErcAction::MINT => "MINT",
            ErcAction::BURN => "BURN",
        }
    }
}

impl<'r> Decode<'r, Postgres> for ErcAction {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        ErcAction::from_str(s).map_err(|_| "Failed to decode ErcAction".into())
    }
}

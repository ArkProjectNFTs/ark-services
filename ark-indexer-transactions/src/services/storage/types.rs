use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum StorageError {
    DatabaseError(String),
    NotFound(String),
    InvalidStatus(String),
    DuplicateToken(String),
    InvalidMintData(String),
    AlreadyExists(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Note the lifetime parameter <'_>
        match self {
            StorageError::DatabaseError(s) => write!(f, "Database error occurred: {s}"),
            StorageError::NotFound(s) => write!(f, "Item not found in storage: {s}"),
            StorageError::InvalidStatus(s) => write!(f, "Invalid status: {s}"),
            StorageError::DuplicateToken(s) => write!(f, "Token already exists in storage: {s}"),
            StorageError::InvalidMintData(s) => write!(f, "Provided mint data is invalid: {s}"),
            StorageError::AlreadyExists(s) => write!(f, "Item already exists in storage: {s}"),
        }
    }
}

impl std::error::Error for StorageError {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Listing,
    CollectionOffer,
    Offer,
    Auction,
    Fulfill,
    Cancelled,
    Executed,
    Sale,
    Mint,
    Burn,
    Transfer,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Mint => write!(f, "MINT"),
            EventType::Burn => write!(f, "BURN"),
            EventType::Transfer => write!(f, "TRANSFER"),
            EventType::Sale => write!(f, "SALE"),
            EventType::Auction => write!(f, "AUCTION"),
            EventType::Listing => write!(f, "LISTING"),
            EventType::CollectionOffer => write!(f, "COLLECTION_OFFER"),
            EventType::Offer => write!(f, "OFFER"),
            EventType::Fulfill => write!(f, "FULFILL"),
            EventType::Cancelled => write!(f, "CANCELLED"),
            EventType::Executed => write!(f, "EXECUTED"),
        }
    }
}

impl FromStr for EventType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MINT" => Ok(EventType::Mint),
            "BURN" => Ok(EventType::Burn),
            "TRANSFER" => Ok(EventType::Transfer),
            "SALE" => Ok(EventType::Sale),
            "AUCTION" => Ok(EventType::Auction),
            "LISTING" => Ok(EventType::Listing),
            "COLLECTION_OFFER" => Ok(EventType::CollectionOffer),
            "OFFER" => Ok(EventType::Offer),
            "FULFILL" => Ok(EventType::Fulfill),
            "CANCELLED" => Ok(EventType::Cancelled),
            "EXECUTED" => Ok(EventType::Executed),
            _ => Err(()),
        }
    }
}

impl Serialize for TokenEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fields_to_serialize = match self {
            TokenEvent::Transfer(event) => {
                let mut map = HashMap::new();
                map.insert("token_event_id", event.token_event_id.clone());
                map.insert("from_address", event.from_address.clone());
                map.insert("to_address", event.to_address.clone());
                map.insert("contract_address", event.contract_address.clone());
                map.insert("transaction_hash", event.transaction_hash.clone());
                map.insert("token_id", event.token_id.clone());
                map.insert("contract_type", event.contract_type.clone());
                map.insert("event_type", "transfer".to_string());
                map.insert("block_timestamp", event.block_timestamp.to_string());
                map.insert(
                    "block_number",
                    event
                        .block_number
                        .map_or("".to_string(), |block_number| block_number.to_string()),
                );
                map.insert("chain_id", event.chain_id.clone());

                map
            }
            TokenEvent::Sale(event) => {
                let mut map = HashMap::new();
                map.insert("token_event_id", event.token_id_hex.clone());
                map.insert("event_type", "sale".to_string());
                map.insert("from_address", event.from_address.clone());
                map.insert("block_timestamp", event.block_timestamp.to_string());
                map.insert("to_address", event.to_address.clone());
                map.insert("nft_contract_address", event.nft_contract_address.clone());
                map.insert(
                    "marketplace_contract_address",
                    event.marketplace_contract_address.clone(),
                );
                map.insert("marketplace_name", event.marketplace_name.clone());
                map.insert("transaction_hash", event.transaction_hash.clone());
                map.insert("token_id", event.token_id.clone());
                map.insert("token_id_hex", event.token_id_hex.clone());
                map.insert("quantity", event.quantity.to_string());
                map.insert("chain_id", event.chain_id.clone());

                if let Some(currency_address) = event.currency_address.clone() {
                    map.insert("currency_address", currency_address);
                }

                map.insert("price", event.price.clone());
                map.insert(
                    "block_number",
                    event
                        .block_number
                        .map_or("".to_string(), |block_number| block_number.to_string()),
                );

                map
            }
        };

        fields_to_serialize.serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum TokenEvent {
    Transfer(TokenTransferEvent),
    Sale(TokenSaleEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenTransferEvent {
    pub token_event_id: String,
    pub block_timestamp: u64,
    pub from_address: String,
    pub to_address: String,
    pub contract_address: String,
    pub chain_id: String,
    pub contract_type: String,
    pub transaction_hash: String,
    pub token_id: String,
    pub token_id_hex: String,
    pub event_type: Option<EventType>,
    pub block_number: Option<u64>,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenSaleEvent {
    pub token_event_id: String,
    pub from_address: String,
    pub to_address: String,
    pub nft_contract_address: String,
    pub nft_type: Option<String>,
    pub marketplace_contract_address: String,
    pub marketplace_name: String,
    pub transaction_hash: String,
    pub chain_id: String,
    pub token_id: String,
    pub token_id_hex: String,
    pub event_type: EventType,
    pub block_timestamp: u64,
    pub block_number: Option<u64>,
    pub updated_at: Option<u64>,
    pub quantity: u64,
    pub currency_address: Option<String>,
    pub price: String,
}

impl Default for TokenTransferEvent {
    fn default() -> Self {
        TokenTransferEvent {
            token_event_id: "0".to_string(),
            block_timestamp: 0,
            from_address: String::new(),
            to_address: String::new(),
            contract_address: String::new(),
            contract_type: String::new(),
            transaction_hash: String::new(),
            token_id: String::new(),
            token_id_hex: String::new(),
            event_type: None,
            block_number: None,
            updated_at: None,
            chain_id: "0x534e5f4d41494e".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TokenInfo {
    pub contract_address: String,
    pub token_id: String,
    pub chain_id: String,
    pub token_id_hex: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TokenMintInfo {
    pub address: String,
    pub block_timestamp: u64,
    pub transaction_hash: String,
    pub block_number: Option<u64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockIndexingStatus {
    None,
    Processing,
    Terminated,
}

impl std::fmt::Display for BlockIndexingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockIndexingStatus::None => write!(f, "None"),
            BlockIndexingStatus::Processing => write!(f, "Processing"),
            BlockIndexingStatus::Terminated => write!(f, "Terminated"),
        }
    }
}

impl FromStr for BlockIndexingStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(BlockIndexingStatus::None),
            "Processing" => Ok(BlockIndexingStatus::Processing),
            "Terminated" => Ok(BlockIndexingStatus::Terminated),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum IndexerStatus {
    Requested,
    Running,
    Stopped,
}

impl fmt::Display for IndexerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexerStatus::Requested => write!(f, "requested"),
            IndexerStatus::Running => write!(f, "running"),
            IndexerStatus::Stopped => write!(f, "stopped"),
        }
    }
}

pub struct Range {
    pub start: u64,
    pub end: u64,
}

pub struct BlockIndexing {
    pub range: Range,
    pub percentage: u64,
    pub status: IndexerStatus,
    pub identifier: String,
    pub indexer_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInfo {
    pub indexer_version: String,
    pub indexer_identifier: String,
    pub block_status: BlockIndexingStatus,
    pub block_number: u64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ContractInfo {
    pub chain_id: String,
    pub contract_address: String,
    pub contract_type: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub image: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_token_event_transfer_serialization() {
        let event = TokenEvent::Transfer(TokenTransferEvent {
            block_timestamp: 1625097600,
            from_address: "0xfrom".to_string(),
            to_address: "0xto".to_string(),
            contract_address: "0xcontract".to_string(),
            contract_type: "ERC721".to_string(),
            transaction_hash: "0xhash".to_string(),
            token_id: "123".to_string(),
            event_type: Some(EventType::Transfer),
            token_event_id: "evt123".to_string(),
            block_number: Some(123),
            updated_at: Some(1625101200),
            chain_id: "0x534e5f4d41494e".to_string(),
            token_id_hex: "0x123".to_string(),
        });

        let serialized = serde_json::to_string(&event).expect("Failed to serialize TokenEvent");

        let expected_json = json!({
            "token_event_id": "evt123",
            "block_number": "123",
            "event_type": "transfer",
            "block_timestamp": "1625097600",
            "from_address": "0xfrom",
            "to_address": "0xto",
            "contract_address": "0xcontract",
            "transaction_hash": "0xhash",
            "token_id": "123",
            "contract_type": "ERC721",
            "chain_id": "0x534e5f4d41494e"
        });

        let expected = expected_json.to_string();

        let serialized_value: Result<Value, _> = serde_json::from_str(&serialized);
        if serialized_value.is_err() {
            println!("`serialized` is not a valid json");
            return;
        }
        let serialized_value = serialized_value.unwrap();

        let expected_value: Result<Value, _> = serde_json::from_str(&expected);
        if expected_value.is_err() {
            println!("`expected` is not a valid json");
            return;
        }
        let expected_value = expected_value.unwrap();

        assert_eq!(serialized_value, expected_value, "json are not equal");
    }
}

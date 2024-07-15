use crate::models::token::TokenEventType;

pub fn event_type_list(values: &[TokenEventType]) -> String {
    let mut types = values
        .iter()
        .map(|v| format!("'{}'", v.to_db_string()))
        .collect::<Vec<_>>();

    if types.contains(&format!("'{}'", TOKEN_EVENT_SALE_STR)) {
        types.push(format!("'{}'", TOKEN_EVENT_EXECUTED_STR));
    }

    types.join(", ")
}

/// DB conversion for TokenEventType
const TOKEN_EVENT_LISTING_STR: &str = "Listing";
const TOKEN_EVENT_COLLECTION_OFFER_STR: &str = "CollectionOffer";
const TOKEN_EVENT_OFFER_STR: &str = "Offer";
const TOKEN_EVENT_AUCTION_STR: &str = "Auction";
const TOKEN_EVENT_FULFILL_STR: &str = "Fulfill";
const TOKEN_EVENT_CANCELLED_STR: &str = "Cancelled";
const TOKEN_EVENT_EXECUTED_STR: &str = "Executed";
const TOKEN_EVENT_SALE_STR: &str = "Sale";
const TOKEN_EVENT_MINT_STR: &str = "Mint";
const TOKEN_EVENT_BURN_STR: &str = "Burn";
const TOKEN_EVENT_TRANSFER_STR: &str = "Transfer";

impl<DB> sqlx::Type<DB> for TokenEventType
where
    DB: sqlx::Database,
    String: sqlx::Type<DB>,
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }
}

impl<'r, DB> sqlx::Decode<'r, DB> for TokenEventType
where
    DB: sqlx::Database,
    &'r str: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<DB>>::decode(value)?;
        match s {
            TOKEN_EVENT_LISTING_STR => Ok(TokenEventType::Listing),
            TOKEN_EVENT_COLLECTION_OFFER_STR => Ok(TokenEventType::CollectionOffer),
            TOKEN_EVENT_OFFER_STR => Ok(TokenEventType::Offer),
            TOKEN_EVENT_AUCTION_STR => Ok(TokenEventType::Auction),
            TOKEN_EVENT_FULFILL_STR => Ok(TokenEventType::Fulfill),
            TOKEN_EVENT_CANCELLED_STR => Ok(TokenEventType::Cancelled),
            TOKEN_EVENT_EXECUTED_STR => Ok(TokenEventType::Executed),
            TOKEN_EVENT_SALE_STR => Ok(TokenEventType::Sale),
            TOKEN_EVENT_MINT_STR => Ok(TokenEventType::Mint),
            TOKEN_EVENT_BURN_STR => Ok(TokenEventType::Burn),
            TOKEN_EVENT_TRANSFER_STR => Ok(TokenEventType::Transfer),
            _ => Err("Invalid event type".into()),
        }
    }
}

/// Convert TokenEventType to matching keys in DB
impl TokenEventType {
    pub fn to_db_string(&self) -> String {
        match self {
            Self::Listing => TOKEN_EVENT_LISTING_STR.to_string(),
            Self::CollectionOffer => TOKEN_EVENT_COLLECTION_OFFER_STR.to_string(),
            Self::Offer => TOKEN_EVENT_OFFER_STR.to_string(),
            Self::Auction => TOKEN_EVENT_AUCTION_STR.to_string(),
            Self::Fulfill => TOKEN_EVENT_FULFILL_STR.to_string(),
            Self::Cancelled => TOKEN_EVENT_CANCELLED_STR.to_string(),
            Self::Executed => TOKEN_EVENT_EXECUTED_STR.to_string(),
            Self::Sale => TOKEN_EVENT_SALE_STR.to_string(),
            Self::Mint => TOKEN_EVENT_MINT_STR.to_string(),
            Self::Burn => TOKEN_EVENT_BURN_STR.to_string(),
            Self::Transfer => TOKEN_EVENT_TRANSFER_STR.to_string(),
        }
    }
}

use crate::models::token::TokenEventType;
use ark_sqlx::providers::marketplace::types::TokenEventType as TokenEventTypeDB;

pub fn event_type_list(values: &[TokenEventType]) -> String {
    let mut types = values
        .iter()
        .map(|v| format!("'{}'", v.to_db_string()))
        .collect::<Vec<_>>();

    if types.contains(&format!("'{}'", TokenEventType::Sale.to_db_string())) {
        types.push(format!("'{}'", TokenEventType::Executed.to_db_string()));
    }

    types.join(", ")
}

/// Convert TokenEventType to matching keys in DB
impl TokenEventType {
    pub fn to_db_string(&self) -> String {
        match self {
            Self::Listing => TokenEventTypeDB::Listing.to_db_string(),
            Self::CollectionOffer => TokenEventTypeDB::CollectionOffer.to_db_string(),
            Self::Offer => TokenEventTypeDB::Offer.to_string(),
            Self::Auction => TokenEventTypeDB::Auction.to_string(),
            Self::Fulfill => TokenEventTypeDB::Fulfill.to_string(),
            Self::Cancelled => TokenEventTypeDB::Cancelled.to_string(),
            Self::Executed => TokenEventTypeDB::Executed.to_string(),
            Self::Sale => TokenEventTypeDB::Sale.to_string(),
            Self::Mint => TokenEventTypeDB::Mint.to_string(),
            Self::Burn => TokenEventTypeDB::Burn.to_string(),
            Self::Transfer => TokenEventType::Transfer.to_string(),
            // Cancel event
            Self::ListingCancelled => TokenEventTypeDB::ListingCancelled.to_string(),
            Self::AuctionCancelled => TokenEventTypeDB::AuctionCancelled.to_string(),
            Self::OfferCancelled => TokenEventTypeDB::OfferCancelled.to_string(),
            Self::ListingExpired => TokenEventTypeDB::ListingExpired.to_string(),
            Self::OfferExpired => TokenEventTypeDB::OfferExpired.to_string(),
        }
    }
}

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
        value: <DB as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        if let Ok(e) = TokenEventTypeDB::decode(value) {
            match e {
                TokenEventTypeDB::Listing => Ok(TokenEventType::Listing),
                TokenEventTypeDB::Auction => Ok(TokenEventType::Auction),
                TokenEventTypeDB::Offer => Ok(TokenEventType::Offer),
                TokenEventTypeDB::CollectionOffer => Ok(TokenEventType::CollectionOffer),
                TokenEventTypeDB::Fulfill => Ok(TokenEventType::Offer),
                TokenEventTypeDB::Executed => Ok(TokenEventType::Executed),
                TokenEventTypeDB::Cancelled => Ok(TokenEventType::Cancelled),
                TokenEventTypeDB::Sale => Ok(TokenEventType::Sale),
                TokenEventTypeDB::Mint => Ok(TokenEventType::Mint),
                TokenEventTypeDB::Burn => Ok(TokenEventType::Burn),
                TokenEventTypeDB::Transfer => Ok(TokenEventType::Transfer),
                TokenEventTypeDB::ListingCancelled => Ok(TokenEventType::ListingCancelled),
                TokenEventTypeDB::AuctionCancelled => Ok(TokenEventType::AuctionCancelled),
                TokenEventTypeDB::OfferCancelled => Ok(TokenEventType::OfferCancelled),
                TokenEventTypeDB::ListingExpired => Ok(TokenEventType::ListingExpired),
                TokenEventTypeDB::OfferExpired => Ok(TokenEventType::OfferExpired),
                TokenEventTypeDB::Rollback => Err("Unsupported rollback event".into()), // _ => Ok(TokenEventType::Burn),
            }
        } else {
            Err("Invalid event type".into())
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenEventType {
    Listing,
    Auction,
    Offer,
    CollectionOffer,
    Fulfill,
    Executed,
    Cancelled,
    Sale,
    Mint,
    Burn,
    Transfer,
    Rollback,
    // Cancel event type
    ListingCancelled,
    AuctionCancelled,
    OfferCancelled,
    // Expired
    ListingExpired,
    OfferExpired,
}

/// DB for EventType
pub(crate) const LISTING_STR: &str = "Listing";
pub(crate) const AUCTION_STR: &str = "Auction";
pub(crate) const OFFER_STR: &str = "Offer";
pub(crate) const COLLECTION_OFFER_STR: &str = "CollectionOffer";
pub(crate) const FULFILL_STR: &str = "Fulfill";
pub(crate) const EXECUTED_STR: &str = "Executed";
pub(crate) const CANCELLED_STR: &str = "Cancelled";
pub(crate) const SALE_STR: &str = "Sale";
pub(crate) const MINT_STR: &str = "Mint";
pub(crate) const BURN_STR: &str = "Burn";
pub(crate) const TRANSFER_STR: &str = "Transfer";
pub(crate) const ROLLBACK_STR: &str = "Rollback";
pub(crate) const LISTING_CANCELLED_STR: &str = "ListingCancelled";
pub(crate) const AUCTION_CANCELLED_STR: &str = "AuctionCancelled";
pub(crate) const OFFER_CANCELLED_STR: &str = "OfferCancelled";
pub(crate) const LISTING_EXPIRED_STR: &str = "ListingExpired";
pub(crate) const OFFER_EXPIRED_STR: &str = "OfferExpired";

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
            LISTING_STR => Ok(TokenEventType::Listing),
            COLLECTION_OFFER_STR => Ok(TokenEventType::CollectionOffer),
            OFFER_STR => Ok(TokenEventType::Offer),
            AUCTION_STR => Ok(TokenEventType::Auction),
            FULFILL_STR => Ok(TokenEventType::Fulfill),
            CANCELLED_STR => Ok(TokenEventType::Cancelled),
            EXECUTED_STR => Ok(TokenEventType::Executed),
            SALE_STR => Ok(TokenEventType::Sale),
            MINT_STR => Ok(TokenEventType::Mint),
            BURN_STR => Ok(TokenEventType::Burn),
            TRANSFER_STR => Ok(TokenEventType::Transfer),
            ROLLBACK_STR => Ok(TokenEventType::Rollback),
            LISTING_CANCELLED_STR => Ok(TokenEventType::ListingCancelled),
            AUCTION_CANCELLED_STR => Ok(TokenEventType::AuctionCancelled),
            OFFER_CANCELLED_STR => Ok(TokenEventType::OfferCancelled),
            LISTING_EXPIRED_STR => Ok(TokenEventType::ListingExpired),
            OFFER_EXPIRED_STR => Ok(TokenEventType::OfferExpired),
            _ => Err("Invalid event type".into()),
        }
    }
}

impl TokenEventType {
    pub fn to_db_string(&self) -> String {
        match self {
            TokenEventType::Listing => LISTING_STR.to_string(),
            TokenEventType::Auction => AUCTION_STR.to_string(),
            TokenEventType::Offer => OFFER_STR.to_string(),
            TokenEventType::CollectionOffer => COLLECTION_OFFER_STR.to_string(),
            TokenEventType::Fulfill => FULFILL_STR.to_string(),
            TokenEventType::Executed => EXECUTED_STR.to_string(),
            TokenEventType::Cancelled => CANCELLED_STR.to_string(),
            TokenEventType::Sale => SALE_STR.to_string(),
            TokenEventType::Mint => MINT_STR.to_string(),
            TokenEventType::Burn => BURN_STR.to_string(),
            TokenEventType::Transfer => TRANSFER_STR.to_string(),
            TokenEventType::Rollback => ROLLBACK_STR.to_string(),
            TokenEventType::ListingCancelled => LISTING_CANCELLED_STR.to_string(),
            TokenEventType::AuctionCancelled => AUCTION_CANCELLED_STR.to_string(),
            TokenEventType::OfferCancelled => OFFER_CANCELLED_STR.to_string(),
            TokenEventType::ListingExpired => LISTING_EXPIRED_STR.to_string(),
            TokenEventType::OfferExpired => OFFER_EXPIRED_STR.to_string(),
        }
    }
}

use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum OfferType {
    Made,
    Received,
    All,
}

impl OfferType {
    pub fn to_sql_condition(&self) -> &str {
        match self {
            OfferType::Made => "token_offer.offer_maker = $2",
            OfferType::Received => "token_offer.to_address = $2",
            OfferType::All => "(token_offer.to_address = $2 OR token_offer.offer_maker = $2)",
        }
    }
}

impl FromStr for OfferType {
    type Err = OfferTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "made" => Ok(OfferType::Made),
            "received" => Ok(OfferType::Received),
            "all" | "" => Ok(OfferType::All),
            _ => Err(OfferTypeParseError(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct OfferTypeParseError(String);

impl fmt::Display for OfferTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid offer type: {}", self.0)
    }
}

impl std::error::Error for OfferTypeParseError {}

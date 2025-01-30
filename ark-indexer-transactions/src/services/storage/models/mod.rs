use bigdecimal::BigDecimal;

pub mod currency;
pub mod transaction_info;

#[derive(sqlx::FromRow)]
pub struct ExistingOrder {
    pub token_address: String,
    pub token_id: Option<String>,
    pub broker_id: String,
    pub start_amount: String,
    pub start_amount_eth: Option<BigDecimal>,
    pub order_type: ExistingOrderType,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "order_type")]
pub enum ExistingOrderType {
    Listing,
    Auction,
    Offer,
    CollectionOffer,
}

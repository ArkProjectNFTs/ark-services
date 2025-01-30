use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug)]
pub struct Currency {
    pub contract_address: String,
    pub chain_id: String,
    pub symbol: String,
    pub decimals: i16,
    pub price_in_usd: BigDecimal,
    pub price_in_eth: BigDecimal,
    pub price_updated_at: DateTime<Utc>,
}

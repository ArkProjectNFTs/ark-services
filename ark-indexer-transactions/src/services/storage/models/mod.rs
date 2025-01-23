use chrono::{DateTime, Utc};

pub mod transaction_info;

#[derive(sqlx::FromRow)]
pub struct Currency {
    pub contract_address: String,
    pub chain_id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub price_in_usd: f64,
    pub price_in_eth: f64,
    pub price_updated_at: DateTime<Utc>,
}

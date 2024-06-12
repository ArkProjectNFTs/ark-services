use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct CollectionData {
    pub address: String,
    pub image: Option<String>,
    pub collection_name: Option<String>,
    pub floor: Option<i32>,
    pub floor_7d_percentage: Option<i32>,
    pub volume_7d_eth: Option<i32>,
    pub top_offer: Option<i64>,
    pub sales_7d: Option<i64>,
    pub marketcap: Option<i32>,
    pub listed_items: Option<i64>,
    pub listed_percentage: Option<i64>,
    pub contract_symbol: Option<String>,
    pub token_count: Option<i64>,
    pub owner_count: Option<i64>,
    pub total_volume: Option<i64>,
    pub total_sales: Option<i64>,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct CollectionPortfolioData {
    pub address: String,
    pub image: Option<String>,
    pub collection_name: Option<String>,
    pub floor: Option<i32>,
    pub token_count: Option<i64>,
}

use arkproject::orderbook;

#[derive(Debug)]
pub struct OrderbookTransactionInfo {
    pub chain_id: String,
    pub tx_hash: String,
    pub event_id: u64,
    pub block_hash: String,
    pub timestamp: u64,
    pub from: String,
    pub event: orderbook::Event,
}

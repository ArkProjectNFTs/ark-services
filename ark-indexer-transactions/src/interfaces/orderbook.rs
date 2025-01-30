use arkproject::orderbook;

#[derive(Debug)]
pub struct OrderTransactionInfo {
    pub chain_id: String,
    pub tx_hash: String,
    pub event_id: u64,
    pub block_hash: String,
    pub timestamp: u64,
    pub from: String,
    pub event: orderbook::Event,
    pub sub_event_id: String,
}

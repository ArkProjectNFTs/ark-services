extern crate openssl;
extern crate openssl_probe;
// Modules déclaration
use warp::Filter;
use std::sync::{Arc, RwLock};
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt};
use serde_json::Value;
use serde::{Serialize};

// Default alocator change
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Debug, Default)]
struct Prices {
    eth_usd: f64,
    strk_usd: f64,
}

#[derive(Serialize)]
struct CombinedPriceResponse {
    eth_usd: f64,
    strk_usd: f64,
}

impl Prices {
    fn new() -> Prices {
        Prices {
            eth_usd: 0.0,
            strk_usd: 0.0,
        }
    }

    fn update_eth_usd(&mut self, price: f64) {
        self.eth_usd = price;
    }

    fn update_strk_usd(&mut self, price: f64) {
        self.strk_usd = price;
    }

    // fn get_eth_usd(&self) -> &f64 {
    //     &self.eth_usd
    // }

    // fn get_strk_usd(&self) -> &f64 {
    //     &self.strk_usd
    // }

    fn get_combined(&self) -> CombinedPriceResponse {
        CombinedPriceResponse {
            eth_usd: self.eth_usd,
            strk_usd: self.strk_usd,
        }
    }
}

#[tokio::main]
async fn main() {
    let prices = Arc::new(RwLock::new(Prices::new()));
    let prices_clone = Arc::clone(&prices);

    tokio::spawn(async move {
        let binance_ws_url = "wss://stream.binance.com:9443/ws/ethusdt@trade/strkusdt@trade";
        let url = url::Url::parse(binance_ws_url).unwrap();

        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

        let (_write, mut read) = ws_stream.split();

        while let Some(Ok(msg)) = read.next().await {
            if let Ok(text) = msg.to_text() {
                if let Ok(value) = serde_json::from_str::<Value>(text) {
                    if let Some(price_str) = value["p"].as_str() {
                        let price: f64 = price_str.parse().unwrap_or(0.0);
                        match value["s"].as_str() {
                            Some("ETHUSDT") => {
                                let mut prices = prices_clone.write().unwrap();
                                prices.update_eth_usd(price);
                            },
                            Some("STRKUSDT") => {
                                let mut prices = prices_clone.write().unwrap();
                                prices.update_strk_usd(price);
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
    });

    let price_filter = warp::any().map(move || Arc::clone(&prices));

    let root_route = warp::path::end()
        .and(price_filter)
        .map(|prices: Arc<RwLock<Prices>>| {
            let prices = prices.read().unwrap();
            let combined_prices = prices.get_combined();
            drop(prices);
            warp::reply::json(&combined_prices)
        });

    let routes = root_route;

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
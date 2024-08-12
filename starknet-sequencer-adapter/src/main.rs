extern crate openssl;
extern crate openssl_probe;
// Modules déclaration
pub mod helpers;
pub mod interfaces;
pub mod services;
// Internal Dependencies definitions
// use helpers::progress_bar::update_progress;
use interfaces::config::Config;
use services::adapter::starknet_adapter::{
    fetch_block, get_latest_block_number, verify_blocks_task,
};
use services::storage::file::{is_block_saved, save_block};
// Standard Dependencies definitions

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
// External Dependencies definitions
use reqwest::Client;
use tokio::sync::{Mutex, Notify};
use tokio::time::{interval, sleep, Duration, Instant};
// Default alocator change
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
pub const BASE_URL: &str =
    "https://alpha-mainnet.starknet.io/feeder_gateway/get_block?blockNumber=";

#[tokio::main]
async fn main() {
    println!("Starting the block ingestion process...");
    match envy::prefixed("SKSQADAPTER_").from_env::<Config>() {
        Ok(config) => {
            let client = Arc::new(Client::new());
            // let start_time = Instant::now();
            let latest_block_number = Arc::new(Mutex::new(0u64));
            let processed_blocks = Arc::new(Mutex::new(0usize));
            let notify = Arc::new(Notify::new());
            let call_interval = Duration::from_millis(60_000 / config.max_calls_per_minute as u64);
            let state_path = Arc::new(PathBuf::from("/opt/fast-indexer/state/state.json"));
            // let events_state_path = Arc::new(PathBuf::from("/opt/fast-indexer/state/events_state.json"));

            // Ensure the state and events directories exist
            fs::create_dir_all("/opt/fast-indexer/state").unwrap();
            fs::create_dir_all("/opt/fast-indexer/events").unwrap();

            let initial_processed_blocks = {
                let mut count = 0;
                for entry in fs::read_dir("/opt/fast-indexer/blocks").unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir() {
                        for block_file in fs::read_dir(path).unwrap() {
                            let block_file = block_file.unwrap();
                            if block_file
                                .file_name()
                                .to_str()
                                .unwrap()
                                .starts_with("block_")
                            {
                                count += 1;
                            }
                        }
                    }
                }
                count
            };

            *processed_blocks.lock().await = initial_processed_blocks;

            let (tx, rx) = tokio::sync::mpsc::channel(100);
            let rx = Arc::new(Mutex::new(rx));

            for i in 0..config.monitor_threads {
                let client = Arc::clone(&client);
                let latest_block_number = Arc::clone(&latest_block_number);
                let tx = tx.clone();
                let _notify = Arc::clone(&notify);

                tokio::spawn(async move {
                    loop {
                        let latest_block_number_value =
                            match get_latest_block_number(BASE_URL, &client).await {
                                Ok(number) => number,
                                Err(e) => {
                                    eprintln!("Failed to get latest block number: {}", e);
                                    sleep(Duration::from_secs(10)).await;
                                    continue;
                                }
                            };

                        // println!("Latest block number: {}", latest_block_number_value);
                        *latest_block_number.lock().await = latest_block_number_value;

                        let range_start =
                            i as u64 * (latest_block_number_value / config.monitor_threads as u64);
                        let range_end = ((i + 1) as u64
                            * (latest_block_number_value / config.monitor_threads as u64))
                            .min(latest_block_number_value);
                        // drop(latest_block_number);
                        // println!("check with range {} to {}", range_start, range_end);
                        for block_number in range_start..=range_end {
                            if !is_block_saved(config.blocks_per_file, block_number) {
                                println!("send to save {}", block_number);
                                tx.send(block_number).await.unwrap();
                            }
                        }

                        // notify.notified().await;
                        sleep(call_interval).await; // Reduce the sleep time to ensure continuous monitoring
                    }
                });
            }

            // Worker thread
            {
                let client = Arc::clone(&client);
                let processed_blocks = Arc::clone(&processed_blocks);
                //let latest_block_number = Arc::clone(&latest_block_number);
                let rx = Arc::clone(&rx);
                let notify = Arc::clone(&notify);

                tokio::spawn(async move {
                    let mut last_update = Instant::now();
                    // let mut blocks_last_minute = 0;
                    let mut interval = interval(call_interval);
                    loop {
                        interval.tick().await;
                        // println!("check new block at : {:?}", interval);
                        let block_number = {
                            let mut rx = rx.lock().await;
                            rx.recv().await.unwrap()
                        };

                        match fetch_block(BASE_URL, &client, block_number).await {
                            Ok(block) => {
                                if let Err(e) =
                                    save_block(config.blocks_per_file, block_number, &block)
                                {
                                    eprintln!("Failed to save block {}: {}", block_number, e);
                                } else {
                                    println!("block: {} saved", block_number);
                                    // let latest_block_number = *latest_block_number.lock().await;
                                    let mut processed_blocks = processed_blocks.lock().await;
                                    *processed_blocks += 1;
                                    // blocks_last_minute += 1;
                                    if last_update.elapsed().as_secs() >= 1 {
                                        last_update = Instant::now();
                                        // blocks_last_minute = 0;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to fetch block {}: {}", block_number, e);
                                notify.notify_one(); // Retry the block
                            }
                        }
                    }
                });
            }

            // Task to verify block format
            {
                let state_path = Arc::clone(&state_path);
                tokio::spawn(async move {
                    verify_blocks_task(state_path).await;
                });
            }

            // // Task to extract events
            // {
            //     let events_state_path = Arc::clone(&events_state_path);
            //     tokio::spawn(async move {
            //         extract_events_task(config.blocks_per_file, events_state_path).await;
            //     });
            // }

            loop {
                sleep(Duration::from_secs(2)).await;
            }
        }
        Err(error) => panic!("{:#?}", error),
    }
}

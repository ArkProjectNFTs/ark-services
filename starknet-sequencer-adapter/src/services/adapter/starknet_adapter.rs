use crate::interfaces::{error::ArkError, event::EventMap};
use crate::services::storage::file::verify_block_format;

use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use rayon::prelude::*;

pub async fn get_latest_block_number(
    base_url: &str,
    client: &Client,
) -> Result<u64, Box<dyn Error + Send>> {
    let url = format!("{}{}", base_url, "latest");
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    let json: Value = response
        .json()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    let block_number = json["block_number"].as_u64().ok_or_else(|| {
        let err: Box<dyn Error + Send> = Box::new(ArkError("Invalid block number".to_string()));
        err
    })?;
    Ok(block_number)
}

pub async fn fetch_block(
    base_url: &str,
    client: &Client,
    block_number: u64,
) -> Result<Value, Box<dyn Error + Send>> {
    let url = format!("{}{}", base_url, block_number);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    let json: Value = response
        .json()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    Ok(json)
}

pub fn extract_events_from_block(block: &Value) -> Vec<(&str, EventMap<'_>)> {
    let transaction_receipts = match block.get("transaction_receipts").and_then(|r| r.as_array()) {
        Some(receipts) => receipts,
        None => return Vec::new(),
    };

    let event_list = transaction_receipts
        .par_iter()
        .filter_map(|transaction| {
            let transaction_hash = transaction.get("transaction_hash")?.as_str()?;

            let events = transaction
                .get("events")
                .and_then(|e| e.as_array())
                .map(|e| e.as_slice())
                .unwrap_or(&[]);

            let transaction_index = transaction.get("transaction_index");
            let execution_status = transaction.get("execution_status");

            Some((
                transaction_hash,
                EventMap {
                    transaction_index,
                    execution_status,
                    events,
                },
            ))
        })
        .collect();

    event_list
}

pub async fn verify_blocks_task(state_path: Arc<PathBuf>) {
    let mut state = HashMap::new();

    // Load state from file
    if state_path.exists() {
        let state_file = File::open(&*state_path).unwrap();
        state = serde_json::from_reader(state_file).unwrap();
    }

    loop {
        let mut verified_blocks = state.clone();

        for entry in fs::read_dir("/opt/fast-indexer/blocks").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                for block_file in fs::read_dir(path).unwrap() {
                    let block_file = block_file.unwrap();
                    let block_path = block_file.path();
                    if let Some(file_name) = block_file.file_name().to_str() {
                        if file_name.starts_with("block_") {
                            if let Ok(block_number) = file_name
                                .trim_start_matches("block_")
                                .trim_end_matches(".json")
                                .parse::<u64>()
                            {
                                if !verified_blocks.contains_key(&block_number) {
                                    match verify_block_format(&block_path) {
                                        Ok(block_number) => {
                                            verified_blocks.insert(block_number, true);
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Failed to verify block format for {:?}: {}",
                                                block_path, e
                                            );
                                        }
                                    }
                                }
                            } else {
                                eprintln!(
                                    "Failed to parse block number from file name: {:?}",
                                    file_name
                                );
                            }
                        }
                    }
                }
            }
        }

        state = verified_blocks.clone();

        // Save state to file
        fs::create_dir_all(state_path.parent().unwrap()).unwrap();
        let state_file = File::create(&*state_path).unwrap();
        serde_json::to_writer(state_file, &state).unwrap();

        sleep(Duration::from_secs(60)).await; // Check every 60 seconds
    }
}

pub async fn extract_events_task(blocks_per_file: u64, state_path: Arc<PathBuf>) {
    let mut state = HashMap::new();

    // Load state from file
    if state_path.exists() {
        let state_file = File::open(&*state_path).unwrap();
        state = serde_json::from_reader(state_file).unwrap();
    }

    loop {
        let mut extracted_blocks = state.clone();

        for entry in fs::read_dir("/opt/fast-indexer/blocks").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                for block_file in fs::read_dir(path).unwrap() {
                    let block_file = block_file.unwrap();
                    let block_path = block_file.path();
                    if let Some(file_name) = block_file.file_name().to_str() {
                        if file_name.starts_with("block_") {
                            if let Ok(block_number) = file_name
                                .trim_start_matches("block_")
                                .trim_end_matches(".json")
                                .parse::<u64>()
                            {
                                if !extracted_blocks.contains_key(&block_number) {
                                    match verify_block_format(&block_path) {
                                        Ok(block_number) => {
                                            let block_content: Value = serde_json::from_str(
                                                &fs::read_to_string(block_path).unwrap(),
                                            )
                                            .unwrap();
                                            let events = extract_events_from_block(&block_content);
                                            for (i,(tx_hash, event)) in events.iter().enumerate() {

                                                let folder = format!(
                                                    "/opt/fast-indexer/events/{}/",
                                                    block_number / blocks_per_file
                                                );
                                                fs::create_dir_all(&folder).unwrap();
                                                let file_path = format!(
                                                    "{}event_{}_{}_{}.json",
                                                    folder, block_number, tx_hash, i
                                                );
                                                
                                                let mut file = File::create(file_path).unwrap();

                                                // Utilisation de serde_json pour sérialiser en JSON
                                                let json_content = serde_json::to_string(&event).unwrap();
                                                file.write_all(json_content.as_bytes()).unwrap();
                                            }
                                            extracted_blocks.insert(block_number, true);
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Failed to verify block format for {:?}: {}",
                                                block_path, e
                                            );
                                        }
                                    }
                                }
                            } else {
                                eprintln!(
                                    "Failed to parse block number from file name: {:?}",
                                    file_name
                                );
                            }
                        }
                    }
                }
            }
        }

        state = extracted_blocks.clone();

        // Save state to file
        fs::create_dir_all(state_path.parent().unwrap()).unwrap();
        let state_file = File::create(&*state_path).unwrap();
        serde_json::to_writer(state_file, &state).unwrap();

        sleep(Duration::from_secs(60)).await; // Check every 60 seconds
    }
}

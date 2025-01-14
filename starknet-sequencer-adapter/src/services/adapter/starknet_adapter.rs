use crate::services::storage::file::verify_block_format;
use crate::{interfaces::error::ArkError, services::state::manager::StateManager};

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

pub fn extract_events_from_block(_block: &Value) -> Vec<Value> {
    // let transaction_receipts = block["transaction_receipts"].as_array();
    let event_list = vec![];
    // transaction_receipts.iter().map(|transaction| {
    //     if let Some(events) = transaction["events"].as_array() {
    //         event_list.push(events)
    //     }
    // });
    event_list
}

pub async fn verify_blocks_task(storage_dir: &str, _state_path: Arc<PathBuf>) {
    // let mut state = HashMap::new();
    let state_manager = StateManager::new(format!("{}/state", storage_dir)).unwrap();

    // Load state from file
    // if state_path.exists() {
    //     let state_file = File::open(&*state_path).unwrap();
    //     state = serde_json::from_reader(state_file).unwrap();
    // }

    loop {
        // let mut verified_blocks = state.clone();

        for entry in fs::read_dir(format!("{}/blocks", storage_dir)).unwrap() {
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
                                .parse::<usize>()
                            {
                                match state_manager.get_block_state(block_number) {
                                    Ok(state) => match verify_block_format(&block_path) {
                                        Ok(_block_number_verified) => {
                                            if !state {
                                                state_manager
                                                    .set_block_state(block_number, true)
                                                    .unwrap();
                                            }
                                        }
                                        Err(e) => {
                                            state_manager
                                                .set_block_state(block_number, false)
                                                .unwrap();
                                            eprintln!(
                                                "Failed to verify block format for {:?}: {}",
                                                block_path, e
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        state_manager.set_block_state(block_number, false).unwrap();
                                        eprintln!(
                                            "Failed to verify state for {:?}: {}",
                                            block_path, e
                                        );
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

        // state = verified_blocks.clone();

        // Save state to file
        // fs::create_dir_all(state_path.parent().unwrap()).unwrap();
        // let state_file = File::create(&*state_path).unwrap();
        // serde_json::to_writer(state_file, &state).unwrap();

        sleep(Duration::from_secs(60)).await; // Check every 60 seconds
    }
}

pub async fn extract_events_task(
    storage_dir: &str,
    blocks_per_file: u64,
    state_path: Arc<PathBuf>,
) {
    let mut state = HashMap::new();

    // Load state from file
    if state_path.exists() {
        let state_file = File::open(&*state_path).unwrap();
        state = serde_json::from_reader(state_file).unwrap();
    }

    loop {
        let mut extracted_blocks = state.clone();

        for entry in fs::read_dir(format!("{}/blocks", storage_dir)).unwrap() {
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
                                            for event in events {
                                                let folder = format!(
                                                    "{}/events/{}/",
                                                    storage_dir,
                                                    block_number / blocks_per_file
                                                );
                                                fs::create_dir_all(&folder).unwrap();
                                                let file_path = format!(
                                                    "{}event_{}_{}.json",
                                                    folder, block_number, event["id"]
                                                );
                                                let mut file = File::create(file_path).unwrap();
                                                file.write_all(event.to_string().as_bytes())
                                                    .unwrap();
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

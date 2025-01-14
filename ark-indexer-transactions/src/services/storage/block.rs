use serde::Deserialize;
use serde_json::Value;
use starknet::providers::sequencer::models::Block;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct BlockWrapper {
    pub block: Block,
}

pub fn get_latest_folder_path(base_path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut max_folder_number = 0;
    let mut latest_folder_path = PathBuf::new();

    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        if entry.path().is_dir() {
            if let Some(folder_name) = entry.file_name().to_str() {
                if let Ok(folder_number) = folder_name.parse::<u64>() {
                    if folder_number > max_folder_number {
                        max_folder_number = folder_number;
                        latest_folder_path = entry.path();
                    }
                }
            }
        }
    }

    Ok(latest_folder_path)
}

pub fn get_latest_block_in_folder(
    folder_path: &PathBuf,
) -> Result<u64, Box<dyn std::error::Error>> {
    let mut max_block_number = 0;

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.starts_with("block_") && file_name.ends_with(".json") {
                if let Some(block_number_str) = file_name
                    .strip_prefix("block_")
                    .and_then(|s| s.strip_suffix(".json"))
                {
                    if let Ok(block_number) = block_number_str.parse::<u64>() {
                        if block_number > max_block_number {
                            max_block_number = block_number;
                        }
                    }
                }
            }
        }
    }

    Ok(max_block_number)
}

fn get_block_file_path(base_path: &str, block_number: u64) -> PathBuf {
    let folder_number = block_number / 100;
    let file_name = format!("block_{}.json", block_number);
    let path = format!("{}/{}/{}", base_path, folder_number, file_name);
    PathBuf::from(path)
}

pub fn read_block_from_file(
    base_path: &str,
    block_number: u64,
) -> Result<BlockWrapper, Box<dyn std::error::Error>> {
    let path = get_block_file_path(base_path, block_number);
    println!("Block number: {:?} - path: {:?}", block_number, path);
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let value: Value = serde_json::from_reader(reader)?;
    let block: Block = serde_json::from_value(value.clone()).map_err(|e| {
        // Log the error and return it
        println!("Error deserializing block: {:?}, JSON: {}", e, value);
        e
    })?;

    let block_wrapper: BlockWrapper = BlockWrapper { block };

    Ok(block_wrapper)
}

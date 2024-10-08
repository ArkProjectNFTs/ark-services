use crate::interfaces::error::ArkError;
use serde_json::Value;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn save_block(
    storage_dir: &str,
    blocks_per_file: u64,
    block_number: u64,
    block: &Value,
) -> std::io::Result<()> {
    let folder = format!("{}/blocks/{}/", storage_dir, block_number / blocks_per_file);
    fs::create_dir_all(&folder)?;
    let file_path = format!("{}block_{}.json", folder, block_number);
    let mut file = File::create(file_path)?;
    file.write_all(block.to_string().as_bytes())?;
    Ok(())
}

pub fn is_block_saved(storage_dir: &str, blocks_per_file: u64, block_number: u64) -> bool {
    let file_path = format!(
        "{}/blocks/{}/block_{}.json",
        storage_dir,
        block_number / blocks_per_file,
        block_number
    );
    Path::new(&file_path).exists()
}

pub fn verify_block_format(block_path: &Path) -> Result<u64, Box<dyn Error>> {
    let mut file = File::open(block_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let json: Value = serde_json::from_str(&content)?;

    if let Some(block_number) = json["block_number"].as_u64() {
        Ok(block_number)
    } else {
        Err(Box::new(ArkError(
            "Block number missing or invalid".to_string(),
        )))
    }
}

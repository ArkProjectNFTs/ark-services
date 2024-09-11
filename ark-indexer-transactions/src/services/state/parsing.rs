use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct ParsingState {
    pub indexed_blocks: HashMap<u64, bool>,
}

impl Default for ParsingState {
    fn default() -> Self {
        Self::new()
    }
}

impl ParsingState {
    pub fn new() -> Self {
        ParsingState {
            indexed_blocks: HashMap::new(),
        }
    }

    pub fn mark_block_indexed(&mut self, block_number: u64) {
        self.indexed_blocks.insert(block_number, true);
    }

    pub fn is_block_indexed(&self, block_number: u64) -> bool {
        *self.indexed_blocks.get(&block_number).unwrap_or(&false)
    }
}

pub fn load_parsing_state(path: &str) -> Result<ParsingState, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let state: ParsingState = serde_json::from_reader(reader)?;
    Ok(state)
}

pub fn save_parsing_state(
    state: &ParsingState,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, state)?;
    Ok(())
}

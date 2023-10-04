use serde::{Deserialize, Serialize};

/// Data of a block.
#[derive(Serialize, Deserialize, Debug)]
pub struct BlockData {
    pub status: String,
    pub indexer_identifier: String,
    pub indexer_version: String,
}

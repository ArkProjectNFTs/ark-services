use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{convert, ProviderError};

/// Data of a block.
#[derive(Serialize, Deserialize, Debug)]
pub struct BlockData {
    pub status: String,
    pub indexer_identifier: String,
    pub indexer_version: String,
}

impl TryFrom<HashMap<String, AttributeValue>> for BlockData {
    type Error = ProviderError;

    fn try_from(data: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(BlockData {
            status: convert::attr_to_str(&data, "Status")?,
            indexer_identifier: convert::attr_to_str(&data, "IndexerIdentifier")?,
            indexer_version: convert::attr_to_str(&data, "IndexerVersion")?,
        })
    }
}

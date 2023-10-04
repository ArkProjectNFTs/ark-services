use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{convert, ProviderError};

/// Data of a collection.
#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionData {
    pub block_number: u64,
    pub contract_type: String,
    pub contract_address: String,
}

impl TryFrom<HashMap<String, AttributeValue>> for CollectionData {
    type Error = ProviderError;

    fn try_from(data: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(CollectionData {
            block_number: convert::attr_to_u64(&data, "BlockNumber")?,
            contract_type: convert::attr_to_str(&data, "ContractType")?,
            contract_address: convert::attr_to_str(&data, "ContractAddress")?,
        })
    }
}

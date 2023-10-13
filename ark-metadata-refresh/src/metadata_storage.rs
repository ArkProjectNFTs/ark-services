use anyhow::Result;
use arkproject::{
    metadata::{
        storage::Storage,
        types::{StorageError, TokenMetadata},
    },
    starknet::CairoU256,
};
use async_trait::async_trait;
use aws_config::load_from_env;
use aws_sdk_dynamodb::{
    types::{AttributeValue, ReturnConsumedCapacity},
    Client,
};
use starknet::core::types::FieldElement;
use tracing::{info, instrument};

#[derive(Debug)]
pub struct MetadataStorage {
    client: Client,
    table_name: String,
}

impl MetadataStorage {
    pub async fn new(table_name: String) -> Self {
        let config = load_from_env().await;
        let client = Client::new(&config);
        Self { client, table_name }
    }
}

#[async_trait]
impl Storage for MetadataStorage {
    fn register_token_metadata(
        &self,
        contract_address: &FieldElement,
        token_id: CairoU256,
        token_metadata: TokenMetadata,
    ) -> Result<(), StorageError> {
        Err(StorageError::DatabaseError)
    }

    fn has_token_metadata(
        &self,
        contract_address: FieldElement,
        token_id: CairoU256,
    ) -> Result<bool, StorageError> {
        Err(StorageError::DatabaseError)
    }

    fn find_token_ids_without_metadata_in_collection(
        &self,
        contract_address: FieldElement,
    ) -> Result<Vec<CairoU256>, StorageError> {
        info!("find_token_ids_without_metadata_in_collection...");

        Err(StorageError::DatabaseError)
    }

    async fn find_token_ids_without_metadata(
        &self,
    ) -> Result<Vec<(FieldElement, CairoU256)>, StorageError> {
        let query_output = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI5PK-GSI5SK-index") // Assuming your GSI for block association is named GSI4
            .key_condition_expression("GSI5PK = :gsi_pk")
            .expression_attribute_values(
                ":gsi_pk",
                AttributeValue::S(String::from("METADATA#false")),
            )
            .send()
            .await
            .map_err(|_| StorageError::DatabaseError)?;

        if let Some(items) = query_output.items {
            let item = items.get(0).unwrap();
            info!("first item: {:?}", item);
            if let Some(data) = item.get("Data") {
                let data_m = data.as_m().unwrap();
                let contract_address_attribute_value = data_m.get("ContractAddress").unwrap().as_s().unwrap();
                let contract_address = FieldElement::from_hex_be(contract_address_attribute_value);

                let token_id_attribute_value = data_m.get("TokenId").unwrap();

                info!(
                    "contract_address: {:?}, token_id={:?}",
                    contract_address,
                    token_id_attribute_value
                );
            }

            // for item in items {
            //     if let Some(pk) = item.get("PK").cloned() {
            //     }
            // }
        }

        Err(StorageError::DatabaseError)
    }
}

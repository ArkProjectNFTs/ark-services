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
use tracing::{field::Field, info, instrument};

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
    async fn register_token_metadata(
        &self,
        contract_address: &FieldElement,
        token_id: CairoU256,
        token_metadata: TokenMetadata,
    ) -> Result<(), StorageError> {
        
        let pk = format!("TOKEN#0x{:064x}#{}", contract_address, token_id.to_hex());
        info!("pk: {:?}", pk);

        info!("token_metadata: {:?}", token_metadata);

        // let r = self.client
        // .update_item()
        // .table_name(self.table_name.clone())
        // .key("PK".to_string(), AttributeValue::S(pk))
        // .key("SK".to_string(), AttributeValue::S(sk))
        // .update_expression("SET #data.#owner = :owner")
        // .expression_attribute_names("#data", "Data")
        // .expression_attribute_names("#owner", "Owner")
        // .expression_attribute_values(":owner".to_string(), AttributeValue::S(owner.to_string()))
        // .return_values(ReturnValue::AllNew)
        // .return_consumed_capacity(ReturnConsumedCapacity::Total)
        // .send()
        // .await
        // .map_err(|e| ProviderError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn has_token_metadata(
        &self,
        contract_address: FieldElement,
        token_id: CairoU256,
    ) -> Result<bool, StorageError> {
        Err(StorageError::DatabaseError)
    }

    async fn find_token_ids_without_metadata_in_collection(
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
            .index_name("GSI5PK-GSI5SK-index")
            .key_condition_expression("GSI5PK = :gsi_pk")
            .expression_attribute_values(
                ":gsi_pk",
                AttributeValue::S(String::from("METADATA#false")),
            )
            .send()
            .await
            .map_err(|_| StorageError::DatabaseError)?;

        let mut results: Vec<(FieldElement, CairoU256)> = Vec::new();

        if let Some(items) = query_output.items {
            for item in items.iter() {
                info!("item: {:?}", item);

                if let Some(data) = item.get("Data") {
                    if data.is_m() {
                        let data_m = data.as_m().unwrap();
                        if let Some(AttributeValue::S(contract_address_attribute_value)) =
                            data_m.get("ContractAddress")
                        {
                            match FieldElement::from_hex_be(contract_address_attribute_value) {
                                Ok(contract_address) => {
                                    if let Some(AttributeValue::S(token_id_attribute_value)) =
                                        data_m.get("TokenIdHex")
                                    {
                                        info!(
                                            "token_id_attribute_value: {:?}",
                                            token_id_attribute_value
                                        );
                                        match CairoU256::from_hex_be(token_id_attribute_value) {
                                            Ok(token_id) => {
                                                info!("token_id: {:?}", token_id);
                                                results.push((contract_address, token_id));
                                            }
                                            Err(_) => continue,
                                        };
                                    }
                                }
                                Err(_) => continue,
                            };
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

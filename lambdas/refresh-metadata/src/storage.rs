use arkproject::{
    metadata::{
        storage::Storage,
        types::{StorageError, TokenMetadata},
    },
    starknet::CairoU256,
};
use starknet::core::types::FieldElement;

#[derive(Default)]
pub struct DynamoStorage {}

impl Storage for DynamoStorage {
    fn register_token_metadata(
        &self,
        contract_address: &FieldElement,
        token_id: CairoU256,
        token_metadata: TokenMetadata,
    ) -> Result<(), StorageError> {
        todo!()
    }

    fn has_token_metadata(
        &self,
        contract_address: FieldElement,
        token_id: CairoU256,
    ) -> Result<bool, StorageError> {
        todo!()
    }

    fn find_token_ids_without_metadata_in_collection(
        &self,
        contract_address: FieldElement,
    ) -> Result<Vec<CairoU256>, StorageError> {
        todo!()
    }
}

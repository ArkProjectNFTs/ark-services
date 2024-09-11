use crate::{
    helpers::common::felt_to_strk_string,
    interfaces::{
        contract::{
            ContractType, ERC1155Event, ERC1400Event, ERC20Event, ERC721Event, NFTInfo,
            StarknetClientError, TransactionInfo,
        },
        event::EventType,
    },
    services::storage::Storage,
};
use starknet::{
    core::types::{BlockId::Hash, Felt},
    providers::Provider,
};

use starknet::providers::sequencer::models::Event;

use super::manager::ContractManager;

impl<S: Storage + Send + Sync, P: Provider + Send + Sync> ContractManager<S, P> {
    pub async fn handle_erc20_event(
        &self,
        event: Event,
        event_id: u64,
        _chain_id: Felt,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contract_origin = event.from_address;
        if let Some((erc_event, erc_compliance)) = self.decode_erc20_event(event)? {
            match erc_event {
                ERC20Event::Transfer { from, to, value } => {
                    let call_data = vec![];

                    let _name = match self
                        .get_contract_property_string(
                            contract_origin,
                            "name",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(name) => name,
                        Err(_) => "".to_owned(),
                    };

                    // println!("Name: {:?}", name);

                    let _symbol = match self
                        .get_contract_property_string(
                            contract_origin,
                            "symbol",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(symbol) => symbol,
                        Err(_) => "".to_owned(),
                    };

                    // println!("Symbol: {:?}", symbol);

                    let _decimals = match self
                        .get_contract_property_string(
                            contract_origin,
                            "decimals",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(decimals) => decimals,
                        Err(_) => "".to_owned(),
                    };

                    // println!("Decimals: {:?}", decimals);

                    // let value = match value {
                    //     Some(value) => value.to_bigint().to_string(),
                    //     None => "0".to_owned(),
                    // };
                    let action = self.detect_erc_action(from, to);
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        from: felt_to_strk_string(from),
                        to: felt_to_strk_string(to),
                        event_type: EventType::Transfer,
                        compliance: erc_compliance,
                        value: Some(value),
                        timestamp: block_timestamp,
                        token_id: None,
                        contract_address: felt_to_strk_string(contract_origin),
                        contract_type: ContractType::ERC20,
                        block_hash: felt_to_strk_string(block_hash),
                        action,
                    };

                    let storage = self.storage.lock().await;
                    storage.store_transaction_info(tx_info).await?;
                    drop(storage);
                }
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    pub async fn handle_erc721_event(
        &self,
        event: Event,
        event_id: u64,
        chain_id: Felt,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contract_origin = event.from_address;
        // println!("block_hash: {:?} - tx_hash: {:?} \n", block_hash, tx_hash);
        if let Some((erc_event, erc_compliance)) = self.decode_erc721_event(event)? {
            match erc_event {
                ERC721Event::Transfer { from, to, token_id } => {
                    // let contract_address = transfer_info.from;
                    let call_data = vec![Felt::from_dec_str(&token_id.to_string())?];
                    let name = match self
                        .get_contract_property_string(
                            contract_origin,
                            "name",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(name) => name,
                        Err(e) => match e {
                            StarknetClientError::EntrypointNotFound(_) => {
                                match self
                                    .get_contract_property_string(
                                        contract_origin,
                                        "Name",
                                        call_data.clone(),
                                        Hash(block_hash),
                                    )
                                    .await
                                {
                                    Ok(alt_name) => alt_name,
                                    Err(_e) => "".to_owned(),
                                }
                            }
                            _ => "".to_owned(),
                        },
                    };
                    // println!("Name: {:?}", name);
                    let symbol = match self
                        .get_contract_property_string(
                            contract_origin,
                            "symbol",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(name) => name,
                        Err(e) => match e {
                            StarknetClientError::EntrypointNotFound(_) => {
                                match self
                                    .get_contract_property_string(
                                        contract_origin,
                                        "Symbol",
                                        call_data.clone(),
                                        Hash(block_hash),
                                    )
                                    .await
                                {
                                    Ok(alt_name) => alt_name,
                                    Err(_e) => "".to_owned(),
                                }
                            }
                            _ => "".to_owned(),
                        },
                    };
                    // println!("Symbol: {:?}", symbol);
                    let metadata_uri = match self
                        .get_contract_property_string(
                            contract_origin,
                            "tokenURI",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(name) => name,
                        Err(e) => match e {
                            StarknetClientError::EntrypointNotFound(_) => {
                                match self
                                    .get_contract_property_string(
                                        contract_origin,
                                        "token_uri",
                                        call_data.clone(),
                                        Hash(block_hash),
                                    )
                                    .await
                                {
                                    Ok(alt_name) => alt_name,
                                    Err(_e) => {
                                        match self
                                            .get_contract_property_string(
                                                contract_origin,
                                                "uri",
                                                call_data.clone(),
                                                Hash(block_hash),
                                            )
                                            .await
                                        {
                                            Ok(alt_alt_name) => alt_alt_name,
                                            Err(_e) => "".to_owned(),
                                        }
                                    }
                                }
                            }
                            _ => "".to_owned(),
                        },
                    };
                    // println!("Meta data URI: {:?}", metadata_uri);
                    let nft_info = NFTInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        contract_address: felt_to_strk_string(contract_origin),
                        token_id: token_id.clone(),
                        name: Some(name),
                        symbol: Some(symbol),
                        metadata_uri: Some(metadata_uri),
                        owner: felt_to_strk_string(to),
                        chain_id: felt_to_strk_string(chain_id),
                        block_hash: felt_to_strk_string(block_hash),
                    };
                    // println!("Found NFT: {:?}", nft_info);
                    let action = self.detect_erc_action(from, to);
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        from: felt_to_strk_string(from),
                        to: felt_to_strk_string(to),
                        value: None,
                        timestamp: block_timestamp,
                        token_id: Some(token_id),
                        contract_address: felt_to_strk_string(contract_origin),
                        contract_type: ContractType::ERC721,
                        block_hash: felt_to_strk_string(block_hash),
                        event_type: EventType::Transfer,
                        compliance: erc_compliance,
                        action,
                    };
                    let storage = self.storage.lock().await;
                    storage.store_nft_info(nft_info).await?;
                    storage.store_transaction_info(tx_info).await?;
                    drop(storage);
                }
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    pub async fn handle_erc1400_event(
        &self,
        event: Event,
        event_id: u64,
        _chain_id: Felt,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contract_origin = event.from_address;
        if let Some((erc_event, erc_compliance)) = self.decode_erc1400_event(event)? {
            match erc_event {
                ERC1400Event::Transfer { from, to, value } => {
                    let call_data = vec![];
                    // if let Some(token_id) = transfer_info.clone().token_id {
                    //     call_data.push(Felt::from_hex(&token_id.clone())?);
                    //     println!("Token ID: {:?}", token_id);
                    // }

                    let _name = match self
                        .get_contract_property_string(
                            contract_origin,
                            "name",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(name) => name,
                        Err(_) => "".to_owned(),
                    };

                    // println!("Name: {:?}", name);

                    let _symbol = match self
                        .get_contract_property_string(
                            contract_origin,
                            "symbol",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(symbol) => symbol,
                        Err(_) => "".to_owned(),
                    };

                    // println!("Symbol: {:?}", symbol);

                    // let value = match transfer_info.amount {
                    //     Some(value) => value.to_bigint().to_string(),
                    //     None => "0".to_owned(),
                    // };

                    let action = self.detect_erc_action(from, to);
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        from: felt_to_strk_string(from),
                        to: felt_to_strk_string(to),
                        value: Some(value),
                        timestamp: block_timestamp,
                        token_id: None,
                        contract_address: felt_to_strk_string(contract_origin),
                        contract_type: ContractType::ERC1400,
                        block_hash: felt_to_strk_string(block_hash),
                        event_type: EventType::Transfer,
                        compliance: erc_compliance,
                        action,
                    };
                    let storage = self.storage.lock().await;
                    storage.store_transaction_info(tx_info).await?;
                    drop(storage);
                }
            }
        }
        Ok(())
    }

    pub async fn handle_erc1155_event(
        &self,
        event: Event,
        event_id: u64,
        chain_id: Felt,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contract_origin = event.from_address;
        if let Some((erc_event, erc_compliance)) = self.decode_erc1155_event(event)? {
            match erc_event {
                ERC1155Event::TransferSingle {
                    operator: _,
                    from,
                    to,
                    id,
                    value,
                } => {
                    let call_data = vec![Felt::from_dec_str(&id.to_string())?];
                    // if let Some(token_id) = transfer_info.clone().token_id {
                    //     call_data.push(Felt::from_hex(&id)?);
                    //     println!("Token ID: {:?}", token_id);
                    // }

                    let uri = match self
                        .get_contract_property_string(
                            contract_origin,
                            "uri",
                            call_data.clone(),
                            Hash(block_hash),
                        )
                        .await
                    {
                        Ok(uri) => uri,
                        Err(_) => "".to_owned(),
                    };

                    // println!("URI: {:?}", uri);
                    let nft_info = NFTInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        contract_address: felt_to_strk_string(contract_origin),
                        token_id: id.clone(),
                        name: None,
                        symbol: None,
                        metadata_uri: Some(uri),
                        owner: felt_to_strk_string(to),
                        chain_id: felt_to_strk_string(chain_id),
                        block_hash: felt_to_strk_string(block_hash),
                    };
                    // let value = match transfer_info.amount {
                    //     Some(value) => value.to_bigint().to_string(),
                    //     None => "0".to_owned(),
                    // };

                    let action = self.detect_erc_action(from, to);
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        from: felt_to_strk_string(from),
                        to: felt_to_strk_string(to),
                        value: Some(value),
                        timestamp: block_timestamp,
                        token_id: Some(id),
                        contract_address: felt_to_strk_string(contract_origin),
                        contract_type: ContractType::ERC1155,
                        block_hash: felt_to_strk_string(block_hash),
                        event_type: EventType::Transfer,
                        compliance: erc_compliance,
                        action,
                    };
                    let storage = self.storage.lock().await;
                    storage.store_nft_info(nft_info).await?;
                    storage.store_transaction_info(tx_info).await?;
                    drop(storage);
                }
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    pub async fn handle_other_event(
        &self,
        _event: Event,
        _event_id: u64,
        _chain_id: Felt,
        _block_hash: Felt,
        _tx_hash: Felt,
        _block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // println!(
        //     "OTHER EVENT HANDLED\nCHAIN: {}\nEvent : {:?}\n",
        //     chain_id, event
        // );
        Ok(())
    }
}

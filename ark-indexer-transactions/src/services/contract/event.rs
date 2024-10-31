use crate::{
    helpers::common::felt_to_strk_string,
    interfaces::{
        contract::{
            ContractInfo, ContractType, ERC1155Event, ERC1400Event, ERC20Event, ERC721Event,
            NFTInfo, StarknetClientError, TransactionInfo,
        },
        event::EventType,
    },
    services::storage::Storage,
};
use bigdecimal::BigDecimal;
use starknet::{
    core::types::{BlockId::Hash, Felt},
    providers::Provider,
};
use std::str::FromStr;

use starknet::providers::sequencer::models::Event;

use super::{
    common::{detect_erc_action, utils::parse_u256},
    erc1155, erc1400, erc20, erc721,
    manager::ContractManager,
};

impl<S, P> ContractManager<S, P>
where
    S: Storage + Send + Sync + 'static,
    P: Provider + Send + Sync + 'static,
{
    pub async fn handle_erc20_event(
        &self,
        event: Event,
        event_id: u64,
        chain_id: &str,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let contract_origin = event.from_address;
        if let Some((erc_event, erc_compliance)) = erc20::decode(&event)? {
            match erc_event {
                ERC20Event::Transfer { from, to, value } => {
                    let call_data = vec![];

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
                        Err(_) => "".to_owned(),
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
                    let action = detect_erc_action(from, to);
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        chain_id: chain_id.to_owned(),
                        from: felt_to_strk_string(from),
                        to: felt_to_strk_string(to),
                        event_type: EventType::Transfer,
                        compliance: erc_compliance,
                        value: Some(value),
                        timestamp: block_timestamp,
                        token_id: Some(BigDecimal::from_str("0")?),
                        contract_address: felt_to_strk_string(contract_origin),
                        contract_type: ContractType::ERC20,
                        block_hash: felt_to_strk_string(block_hash),
                        action,
                        sub_event_id: format!("{}_O", event_id),
                    };
                    let _nft_info = NFTInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        contract_address: felt_to_strk_string(contract_origin),
                        token_id: None,
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        metadata_uri: None,
                        owner: felt_to_strk_string(to),
                        chain_id: chain_id.to_owned(),
                        block_hash: felt_to_strk_string(block_hash),
                        block_timestamp,
                    };
                    let contract_info = ContractInfo {
                        chain_id: chain_id.to_owned(),
                        contract_type: ContractType::ERC20,
                        contract_address: felt_to_strk_string(contract_origin),
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        image: None,
                    };
                    // println!("TX INFO : {:?}", tx_info);
                    let storage = self.storage.lock().await;
                    storage.store_contract(contract_info).await?;
                    // storage.store_token(nft_info.clone()).await?;
                    // storage.store_token_event(tx_info.clone()).await?;
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
        chain_id: &str,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let contract_origin = event.from_address;
        // println!("block_hash: {:?} - tx_hash: {:?} \n", block_hash, tx_hash);
        if let Some((erc_event, erc_compliance)) = erc721::decode(&event)? {
            match erc_event {
                ERC721Event::Transfer {
                    from,
                    to,
                    token_id_low,
                    token_id_high,
                } => {
                    // let contract_address = transfer_info.from;
                    let call_data = vec![];
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
                    let call_data_uri = vec![token_id_low, token_id_high];
                    let metadata_uri = match self
                        .get_contract_property_string(
                            contract_origin,
                            "tokenURI",
                            call_data_uri.clone(),
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
                                        call_data_uri.clone(),
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
                                                call_data_uri.clone(),
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
                    let contract_info = ContractInfo {
                        chain_id: chain_id.to_owned(),
                        contract_type: ContractType::ERC721,
                        contract_address: felt_to_strk_string(contract_origin),
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        image: None,
                    };
                    // println!("Meta data URI: {:?}", metadata_uri);
                    let token_id: BigDecimal = parse_u256(&token_id_low, &token_id_high);
                    let nft_info = NFTInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        contract_address: felt_to_strk_string(contract_origin),
                        token_id: Some(token_id.clone()),
                        name: Some(name),
                        symbol: Some(symbol),
                        metadata_uri: Some(metadata_uri),
                        owner: felt_to_strk_string(to),
                        chain_id: chain_id.to_owned(),
                        block_hash: felt_to_strk_string(block_hash),
                        block_timestamp,
                    };
                    // println!("Found NFT: {:?}", nft_info);
                    let action = detect_erc_action(from, to);
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        chain_id: chain_id.to_owned(),
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
                        sub_event_id: format!("{}_O", event_id),
                    };
                    let storage = self.storage.lock().await;
                    storage.store_contract(contract_info).await?;
                    storage.store_token(nft_info.clone()).await?;
                    storage.store_token_event(tx_info.clone()).await?;
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
        chain_id: &str,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let contract_origin = event.from_address;
        if let Some((erc_event, erc_compliance)) = erc1400::decode(&event)? {
            match erc_event {
                ERC1400Event::Transfer { from, to, value } => {
                    let call_data = vec![];
                    // if let Some(token_id) = transfer_info.clone().token_id {
                    //     call_data.push(Felt::from_hex(&token_id.clone())?);
                    //     println!("Token ID: {:?}", token_id);
                    // }

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
                        Err(_) => "".to_owned(),
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
                        Ok(symbol) => symbol,
                        Err(_) => "".to_owned(),
                    };

                    // println!("Symbol: {:?}", symbol);

                    // let value = match transfer_info.amount {
                    //     Some(value) => value.to_bigint().to_string(),
                    //     None => "0".to_owned(),
                    // };

                    let action = detect_erc_action(from, to);
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        chain_id: chain_id.to_owned(),
                        from: felt_to_strk_string(from),
                        to: felt_to_strk_string(to),
                        value: Some(value),
                        timestamp: block_timestamp,
                        token_id: Some(BigDecimal::from_str("0")?),
                        contract_address: felt_to_strk_string(contract_origin),
                        contract_type: ContractType::ERC1400,
                        block_hash: felt_to_strk_string(block_hash),
                        event_type: EventType::Transfer,
                        compliance: erc_compliance,
                        action,
                        sub_event_id: format!("{}_O", event_id),
                    };
                    let _nft_info = NFTInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        contract_address: felt_to_strk_string(contract_origin),
                        token_id: None,
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        metadata_uri: None,
                        owner: felt_to_strk_string(to),
                        chain_id: chain_id.to_owned(),
                        block_hash: felt_to_strk_string(block_hash),
                        block_timestamp,
                    };
                    let contract_info = ContractInfo {
                        chain_id: chain_id.to_owned(),
                        contract_type: ContractType::ERC1400,
                        contract_address: felt_to_strk_string(contract_origin),
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        image: None,
                    };
                    let storage = self.storage.lock().await;
                    storage.store_contract(contract_info).await?;
                    // storage.store_token(nft_info.clone()).await?;
                    // storage.store_token_event(tx_info.clone()).await?;
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
        chain_id: &str,
        block_hash: Felt,
        tx_hash: Felt,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let contract_origin = event.from_address;
        if let Some((erc_event, erc_compliance)) = erc1155::decode(&event)? {
            match erc_event {
                ERC1155Event::TransferSingle {
                    operator: _,
                    from,
                    to,
                    id_low,
                    id_high,
                    value,
                } => {
                    let call_data = vec![id_low, id_high];
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
                        Err(_) => "".to_owned(),
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
                        Ok(symbol) => symbol,
                        Err(_) => "".to_owned(),
                    };

                    // println!("URI: {:?}", uri);
                    let token_id = parse_u256(&id_low, &id_high);
                    let nft_info = NFTInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        contract_address: felt_to_strk_string(contract_origin),
                        token_id: Some(token_id.clone()),
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        metadata_uri: Some(uri),
                        owner: felt_to_strk_string(to),
                        chain_id: chain_id.to_owned(),
                        block_hash: felt_to_strk_string(block_hash),
                        block_timestamp,
                    };
                    // let value = match transfer_info.amount {
                    //     Some(value) => value.to_bigint().to_string(),
                    //     None => "0".to_owned(),
                    // };

                    let action = detect_erc_action(from, to);
                    let contract_info = ContractInfo {
                        chain_id: chain_id.to_owned(),
                        contract_type: ContractType::ERC1155,
                        contract_address: felt_to_strk_string(contract_origin),
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        image: None,
                    };
                    let tx_info = TransactionInfo {
                        tx_hash: felt_to_strk_string(tx_hash),
                        event_id,
                        chain_id: chain_id.to_owned(),
                        from: felt_to_strk_string(from),
                        to: felt_to_strk_string(to),
                        value: Some(value),
                        timestamp: block_timestamp,
                        token_id: Some(token_id),
                        contract_address: felt_to_strk_string(contract_origin),
                        contract_type: ContractType::ERC1155,
                        block_hash: felt_to_strk_string(block_hash),
                        event_type: EventType::Transfer,
                        compliance: erc_compliance,
                        action,
                        sub_event_id: format!("{}_O", event_id),
                    };
                    let storage = self.storage.lock().await;
                    storage.store_contract(contract_info).await?;
                    storage.store_token(nft_info.clone()).await?;
                    storage.store_token_event(tx_info.clone()).await?;
                    storage.store_nft_info(nft_info).await?;
                    storage.store_transaction_info(tx_info).await?;
                    drop(storage);
                }
                ERC1155Event::TransferBatch {
                    operator: _,
                    from,
                    to,
                    ids,
                    values,
                } => {
                    let mut nft_infos = Vec::new();
                    let mut tx_infos = Vec::new();
                    let call_data = vec![];

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
                        Err(_) => "".to_owned(),
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
                        Ok(symbol) => symbol,
                        Err(_) => "".to_owned(),
                    };

                    let contract_info = ContractInfo {
                        chain_id: chain_id.to_owned(),
                        contract_type: ContractType::ERC1155,
                        contract_address: felt_to_strk_string(contract_origin),
                        name: Some(name.clone()),
                        symbol: Some(symbol.clone()),
                        image: None,
                    };
                    for (index, ((id_low, id_high), value)) in
                        ids.into_iter().zip(values.iter()).enumerate()
                    {
                        let call_data = vec![id_low, id_high];
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
                        let token_id: BigDecimal = parse_u256(&id_low, &id_high);
                        let nft_info = NFTInfo {
                            tx_hash: felt_to_strk_string(tx_hash),
                            contract_address: felt_to_strk_string(contract_origin),
                            token_id: Some(token_id.clone()),
                            name: None,
                            symbol: None,
                            metadata_uri: Some(uri),
                            owner: felt_to_strk_string(to),
                            chain_id: chain_id.to_owned(),
                            block_hash: felt_to_strk_string(block_hash),
                            block_timestamp,
                        };

                        let action = detect_erc_action(from, to);
                        let tx_info = TransactionInfo {
                            tx_hash: felt_to_strk_string(tx_hash),
                            event_id,
                            chain_id: chain_id.to_owned(),
                            from: felt_to_strk_string(from),
                            to: felt_to_strk_string(to),
                            value: Some(value.clone()),
                            timestamp: block_timestamp,
                            token_id: Some(token_id),
                            contract_address: felt_to_strk_string(contract_origin),
                            contract_type: ContractType::ERC1155,
                            block_hash: felt_to_strk_string(block_hash),
                            event_type: EventType::TransferBatch,
                            compliance: erc_compliance.clone(),
                            action,
                            sub_event_id: format!("{}_{}", event_id, index),
                        };

                        nft_infos.push(nft_info);
                        tx_infos.push(tx_info);
                    }

                    let storage = self.storage.lock().await;
                    storage.store_contract(contract_info).await?;
                    // Utilisation de futures pour paralléliser les opérations d'enregistrement
                    let store_nft_futures = nft_infos
                        .clone()
                        .into_iter()
                        .map(|nft_info| storage.store_nft_info(nft_info));
                    let store_token_futures = nft_infos
                        .into_iter()
                        .map(|nft_info| storage.store_token(nft_info));
                    let store_tx_futures = tx_infos
                        .clone()
                        .into_iter()
                        .map(|tx_info| storage.store_transaction_info(tx_info));
                    let store_te_futures = tx_infos
                        .into_iter()
                        .map(|tx_info| storage.store_token_event(tx_info));
                    // Exécution parallèle des futures
                    let (token_results, nft_results, te_results, tx_results) = tokio::join!(
                        futures::future::join_all(store_token_futures),
                        futures::future::join_all(store_nft_futures),
                        futures::future::join_all(store_te_futures),
                        futures::future::join_all(store_tx_futures)
                    );

                    // Vérification des résultats
                    for result in nft_results
                        .into_iter()
                        .chain(tx_results)
                        .chain(te_results)
                        .chain(token_results)
                    {
                        result?;
                    }

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
        _chain_id: &str,
        _block_hash: Felt,
        _tx_hash: Felt,
        _block_timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // println!(
        //     "OTHER EVENT HANDLED\nCHAIN: {}\nEvent : {:?}\n",
        //     chain_id, event
        // );
        Ok(())
    }
}

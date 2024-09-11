use std::str::FromStr;

use crate::{
    helpers::cairo_string_parser::{parse_cairo_string, ParseError},
    interfaces::contract::{ERC1155Event, ERC1400Event, ERC20Event, ERC721Event},
    services::storage::Storage,
};
use bigdecimal::BigDecimal;
use starknet::{
    core::types::{Felt, U256},
    providers::Provider,
};

use crate::interfaces::event::{self as EventInterface, ERCCompliance};
use starknet::providers::sequencer::models::Event;

use super::manager::ContractManager;
use num_traits::ToPrimitive;

impl<S: Storage + Send + Sync, P: Provider + Send + Sync> ContractManager<S, P> {
    pub fn decode_erc20_event(
        &self,
        event: Event,
    ) -> Result<Option<(ERC20Event, ERCCompliance)>, Box<dyn std::error::Error>> {
        if !event.keys.is_empty() {
            match event.keys[0] {
                key if key == EventInterface::TRANSFER => {
                    if event.keys.len() == 3 && event.data.len() == 2 {
                        // let value = CairoU256{low: event.data[0]., high:event.data[1]}.to_hex();
                        let low = event.data[0].to_u128().unwrap();
                        let high = event.data[1].to_u128().unwrap();
                        let value =
                            BigDecimal::from_str(&U256::from_words(low, high).to_string()).unwrap();
                        Ok(Some((
                            ERC20Event::Transfer {
                                from: event.keys[1],
                                to: event.keys[2],
                                value,
                            },
                            ERCCompliance::OPENZEPPELIN,
                        )))
                    } else {
                        // handle wrong implementation of ERC20 Transfer
                        if event.keys.len() == 1 && event.data.len() == 4 {
                            let from = event.data[0];
                            let to = event.data[1];
                            let low = event.data[2].to_u128().unwrap();
                            let high = event.data[3].to_u128().unwrap();
                            let value =
                                BigDecimal::from_str(&U256::from_words(low, high).to_string())
                                    .unwrap();
                            return Ok(Some((
                                ERC20Event::Transfer { from, to, value },
                                ERCCompliance::OTHER,
                            )));
                        }
                        Ok(None)
                    }
                }
                key if key == EventInterface::APPROVAL => {
                    if event.keys.len() == 3 && event.data.len() == 2 {
                        let value_low = event.data[0].to_u128().unwrap();
                        let value_high = event.data[1].to_u128().unwrap();
                        let value = U256::from_words(value_low, value_high).to_string();
                        Ok(Some((
                            ERC20Event::Approval {
                                owner: event.keys[1],
                                spender: event.keys[2],
                                value,
                            },
                            ERCCompliance::OPENZEPPELIN,
                        )))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn decode_erc721_event(
        &self,
        event: Event,
    ) -> Result<Option<(ERC721Event, ERCCompliance)>, Box<dyn std::error::Error>> {
        // println!("Transfer key {:?}", EventInterface::TRANSFER);
        if !event.keys.is_empty() {
            match event.keys[0] {
                key if key == EventInterface::TRANSFER => {
                    // ?????
                    println!("ERC 721 - Transfert");
                    if event.keys.len() == 5 {
                        let value_low = event.keys[3].to_u128().unwrap();
                        let value_high = event.keys[4].to_u128().unwrap();
                        let token_id = BigDecimal::from_str(
                            &U256::from_words(value_low, value_high).to_string(),
                        )
                        .unwrap();
                        println!("-> Compliance - OPENZEPPELIN Standard");
                        Ok(Some((
                            ERC721Event::Transfer {
                                from: event.keys[1],
                                to: event.keys[2],
                                token_id,
                            },
                            ERCCompliance::OPENZEPPELIN,
                        )))
                    } else if event.keys.len() == 1 && event.data.len() == 4 {
                        let value_low = event.data[2].to_u128().unwrap();
                        let value_high = event.data[3].to_u128().unwrap();
                        let token_id = BigDecimal::from_str(
                            &U256::from_words(value_low, value_high).to_string(),
                        )
                        .unwrap();
                        println!("-> Compliance - Not Standard");
                        Ok(Some((
                            ERC721Event::Transfer {
                                from: event.data[0],
                                to: event.data[1],
                                token_id,
                            },
                            ERCCompliance::OTHER,
                        )))
                    } else {
                        println!("-> Error - False Parsing on Transfer");
                        Ok(None)
                    }
                }
                key if key == EventInterface::APPROVAL => {
                    println!("ERC 721 - Approval");
                    if event.keys.len() == 5 {
                        let value_low = event.keys[3].to_u128().unwrap();
                        let value_high = event.keys[4].to_u128().unwrap();
                        let token_id = BigDecimal::from_str(
                            &U256::from_words(value_low, value_high).to_string(),
                        )
                        .unwrap();
                        println!("-> Compliance - OPENZEPPELIN Standard");
                        return Ok(Some((
                            ERC721Event::Approval {
                                owner: event.keys[1],
                                approved: event.keys[2],
                                token_id,
                            },
                            ERCCompliance::OPENZEPPELIN,
                        )));
                    }
                    Ok(None)
                }
                key if key == EventInterface::APPROVAL_FOR_ALL => {
                    println!("ERC 721 - Approval For all");
                    if event.keys.len() == 3 {
                        if event.data[0] == Felt::ONE {
                            println!("-> Compliance - OPENZEPPELIN Standard");
                            return Ok(Some((
                                ERC721Event::ApprovalForAll {
                                    owner: event.keys[1],
                                    operator: event.keys[2],
                                    approved: true,
                                },
                                ERCCompliance::OPENZEPPELIN,
                            )));
                        } else if event.data[0] == Felt::ZERO {
                            println!("-> Compliance - OPENZEPPELIN Standard");
                            return Ok(Some((
                                ERC721Event::ApprovalForAll {
                                    owner: event.keys[1],
                                    operator: event.keys[2],
                                    approved: false,
                                },
                                ERCCompliance::OPENZEPPELIN,
                            )));
                        } else {
                            println!("-> Compliance - Not Standard");
                            return Ok(None);
                        }
                    }
                    Ok(None)
                }
                _ => {
                    println!("ERC721 EVENT NOT PARSED =>");
                    println!("ERC 721 keys not parsing with data : {:?}", event.data);
                    println!("ERC 721 keys not parsing with keys {:?}", event.keys);
                    Ok(None)
                }
            }
        } else {
            println!(
                "ERC 721 keys empty fail parsing with data : {:?}",
                event.data
            );
            println!("ERC 721 keys empty fail parsing with keys {:?}", event.keys);
            Ok(None)
        }
    }

    pub fn decode_erc1155_event(
        &self,
        event: Event,
    ) -> Result<Option<(ERC1155Event, ERCCompliance)>, Box<dyn std::error::Error>> {
        if !event.keys.is_empty() {
            match event.keys[0] {
                key if key == EventInterface::TRANSFER_SINGLE => {
                    if event.keys.len() == 4 && event.data.len() == 4 {
                        let operator = event.keys[1];
                        let from = event.keys[2];
                        let to = event.keys[3];
                        let id_low = event.data[0].to_u128().unwrap();
                        let id_high = event.data[1].to_u128().unwrap();
                        let id =
                            BigDecimal::from_str(&U256::from_words(id_low, id_high).to_string())
                                .unwrap();
                        let value_low = event.data[2].to_u128().unwrap();
                        let value_high = event.data[3].to_u128().unwrap();
                        let value = BigDecimal::from_str(
                            &U256::from_words(value_low, value_high).to_string(),
                        )
                        .unwrap();
                        Ok(Some((
                            ERC1155Event::TransferSingle {
                                from,
                                to,
                                value,
                                operator,
                                id,
                            },
                            ERCCompliance::OPENZEPPELIN,
                        )))
                    } else if event.keys.len() == 1 && event.data.len() == 7 {
                        let operator = event.data[0];
                        let from = event.data[1];
                        let to = event.data[2];
                        let id_low = event.data[3].to_u128().unwrap();
                        let id_high = event.data[4].to_u128().unwrap();
                        let id =
                            BigDecimal::from_str(&U256::from_words(id_low, id_high).to_string())
                                .unwrap();
                        let value_low = event.data[5].to_u128().unwrap();
                        let value_high = event.data[6].to_u128().unwrap();
                        let value = BigDecimal::from_str(
                            &U256::from_words(value_low, value_high).to_string(),
                        )
                        .unwrap();
                        Ok(Some((
                            ERC1155Event::TransferSingle {
                                from,
                                to,
                                value,
                                operator,
                                id,
                            },
                            ERCCompliance::OTHER,
                        )))
                    } else {
                        Ok(None)
                    }
                }
                key if key == EventInterface::TRANSFER_BATCH => {
                    if event.keys.len() == 4 && event.data.len() >= 4 {
                        let ids_nb_elems_idx = 0;
                        let ids_nb_elems =
                            event.data[ids_nb_elems_idx].to_bigint().to_usize().unwrap();
                        let ids_data_start = ids_nb_elems_idx + 1;
                        let values_nb_elems_idx = ids_data_start + (2 * ids_nb_elems);
                        let values_nb_elems = event.data[values_nb_elems_idx]
                            .to_bigint()
                            .to_usize()
                            .unwrap();
                        let values_data_start = values_nb_elems_idx + 1;
                        if ids_nb_elems == values_nb_elems {
                            let ids = event.data
                                [ids_data_start..(ids_data_start + 2 * ids_nb_elems)]
                                .to_vec();
                            let values = event.data
                                [values_data_start..(values_data_start + 2 * values_nb_elems)]
                                .to_vec();
                            return Ok(Some((
                                ERC1155Event::TransferBatch {
                                    operator: event.keys[1],
                                    from: event.keys[2],
                                    to: event.keys[3],
                                    ids,
                                    values,
                                },
                                ERCCompliance::OPENZEPPELIN,
                            )));
                        } else {
                            return Ok(None);
                        }
                    }
                    Ok(None)
                }
                key if key == EventInterface::APPROVAL_FOR_ALL => {
                    if event.keys.len() == 3 && event.data.len() == 1 {
                        if event.data[0] == Felt::ONE {
                            Ok(Some((
                                ERC1155Event::ApprovalForAll {
                                    owner: event.keys[1],
                                    operator: event.keys[2],
                                    approved: true,
                                },
                                ERCCompliance::OPENZEPPELIN,
                            )))
                        } else if event.data[0] == Felt::ZERO {
                            Ok(Some((
                                ERC1155Event::ApprovalForAll {
                                    owner: event.keys[1],
                                    operator: event.keys[2],
                                    approved: false,
                                },
                                ERCCompliance::OPENZEPPELIN,
                            )))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                key if key == EventInterface::URI => {
                    if event.keys.len() == 3 && event.data.len() > 3 {
                        match parse_cairo_string(event.data) {
                            Ok(value) => {
                                let id_low = event.keys[1].to_u128().unwrap();
                                let id_high = event.keys[2].to_u128().unwrap();
                                let id = U256::from_words(id_low, id_high).to_string();
                                Ok(Some((
                                    ERC1155Event::URI { value, id },
                                    ERCCompliance::OPENZEPPELIN,
                                )))
                            }
                            Err(e) => {
                                match e {
                                    ParseError::ByteArrayError => {
                                        // @todo: Implements wrong implementations on URI
                                        Ok(None)
                                    }
                                    _ => Ok(None),
                                }
                            }
                        }
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    // Decoded like https://github.com/Consensys/UniversalToken/blob/master/contracts/IERC1400.sol
    pub fn decode_erc1400_event(
        &self,
        event: Event,
    ) -> Result<Option<(ERC1400Event, ERCCompliance)>, Box<dyn std::error::Error>> {
        if event.keys.len() >= 3 {
            let from = event.keys[1];
            let to = event.keys[2];
            let value_low = event.data[0].to_u128().unwrap();
            let value_high = event.data[1].to_u128().unwrap();
            let value =
                BigDecimal::from_str(&U256::from_words(value_low, value_high).to_string()).unwrap();
            Ok(Some((
                ERC1400Event::Transfer { from, to, value },
                ERCCompliance::OTHER,
            )))
        } else {
            Ok(None)
        }
    }
}

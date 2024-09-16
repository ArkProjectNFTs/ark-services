use crate::helpers::cairo_string_parser::parse_cairo_string;
use crate::interfaces::contract::ERC1155Event;
use crate::interfaces::event::{self as EventInterface, ERCCompliance};
use crate::services::contract::common::utils::parse_u256;
use num_traits::ToPrimitive;
use starknet::providers::sequencer::models::Event;
use starknet_crypto::Felt;

pub fn decode(
    event: &Event,
) -> Result<Option<(ERC1155Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.is_empty() {
        return Ok(None);
    }

    match event.keys[0] {
        key if key == EventInterface::TRANSFER_SINGLE => decode_transfer_single(event),
        key if key == EventInterface::TRANSFER_BATCH => decode_transfer_batch(event),
        key if key == EventInterface::APPROVAL_FOR_ALL => decode_approval_for_all(event),
        key if key == EventInterface::URI => decode_uri(event),
        _ => Ok(None),
    }
}

fn decode_transfer_single(
    event: &Event,
) -> Result<Option<(ERC1155Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 4 && event.data.len() == 4 {
        let id = parse_u256(&event.data[0], &event.data[1]);
        let value = parse_u256(&event.data[2], &event.data[3]);
        Ok(Some((
            ERC1155Event::TransferSingle {
                operator: event.keys[1],
                from: event.keys[2],
                to: event.keys[3],
                id,
                value,
            },
            ERCCompliance::OPENZEPPELIN,
        )))
    } else if event.keys.len() == 1 && event.data.len() == 7 {
        let id = parse_u256(&event.data[3], &event.data[4]);
        let value = parse_u256(&event.data[5], &event.data[6]);
        Ok(Some((
            ERC1155Event::TransferSingle {
                operator: event.data[0],
                from: event.data[1],
                to: event.data[2],
                id,
                value,
            },
            ERCCompliance::OTHER,
        )))
    } else {
        Ok(None)
    }
}

fn decode_transfer_batch(
    event: &Event,
) -> Result<Option<(ERC1155Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 4 && event.data.len() >= 4 {
        let ids_nb_elems = event.data[0].to_usize().ok_or("Invalid ids_nb_elems")?;
        let values_nb_elems_idx = 1 + (2 * ids_nb_elems);
        let values_nb_elems = event.data[values_nb_elems_idx]
            .to_usize()
            .ok_or("Invalid values_nb_elems")?;

        if ids_nb_elems == values_nb_elems {
            let ids = event.data[1..(1 + 2 * ids_nb_elems)].to_vec();
            let values = event.data[(values_nb_elems_idx + 1)..].to_vec();
            Ok(Some((
                ERC1155Event::TransferBatch {
                    operator: event.keys[1],
                    from: event.keys[2],
                    to: event.keys[3],
                    ids,
                    values,
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

fn decode_approval_for_all(
    event: &Event,
) -> Result<Option<(ERC1155Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 3 && event.data.len() == 1 {
        let approved = event.data[0] == Felt::ONE;
        Ok(Some((
            ERC1155Event::ApprovalForAll {
                owner: event.keys[1],
                operator: event.keys[2],
                approved,
            },
            ERCCompliance::OPENZEPPELIN,
        )))
    } else {
        Ok(None)
    }
}

fn decode_uri(
    event: &Event,
) -> Result<Option<(ERC1155Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 3 && event.data.len() > 3 {
        if let Ok(value) = parse_cairo_string(event.data.clone()) {
            let id = parse_u256(&event.keys[1], &event.keys[2]).to_string();
            Ok(Some((
                ERC1155Event::URI { value, id },
                ERCCompliance::OPENZEPPELIN,
            )))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

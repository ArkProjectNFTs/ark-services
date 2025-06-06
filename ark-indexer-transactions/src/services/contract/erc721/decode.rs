use crate::interfaces::contract::ERC721Event;
use crate::interfaces::event::{self as EventInterface, ERCCompliance};
use starknet::core::types::Felt;
use starknet::providers::sequencer::models::Event;

pub fn decode(
    event: &Event,
) -> Result<Option<(ERC721Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.is_empty() {
        return Ok(None);
    }

    match event.keys[0] {
        key if key == EventInterface::TRANSFER => decode_transfer(event),
        key if key == EventInterface::APPROVAL => decode_approval(event),
        key if key == EventInterface::APPROVAL_FOR_ALL => decode_approval_for_all(event),
        _ => Ok(None),
    }
}

fn decode_transfer(
    event: &Event,
) -> Result<Option<(ERC721Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 5 {
        Ok(Some((
            ERC721Event::Transfer {
                from: event.keys[1],
                to: event.keys[2],
                token_id_low: event.keys[3],
                token_id_high: event.keys[4],
            },
            ERCCompliance::OPENZEPPELIN,
        )))
    } else if event.keys.len() == 1 && event.data.len() == 4 {
        Ok(Some((
            ERC721Event::Transfer {
                from: event.data[0],
                to: event.data[1],
                token_id_low: event.data[2],
                token_id_high: event.data[3],
            },
            ERCCompliance::OTHER,
        )))
    } else {
        Ok(None)
    }
}

fn decode_approval(
    event: &Event,
) -> Result<Option<(ERC721Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 5 {
        Ok(Some((
            ERC721Event::Approval {
                owner: event.keys[1],
                approved: event.keys[2],
                token_id_low: event.keys[3],
                token_id_high: event.keys[4],
            },
            ERCCompliance::OPENZEPPELIN,
        )))
    } else {
        Ok(None)
    }
}

fn decode_approval_for_all(
    event: &Event,
) -> Result<Option<(ERC721Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 3 && event.data.len() == 1 {
        let approved = event.data[0] == Felt::ONE;
        Ok(Some((
            ERC721Event::ApprovalForAll {
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

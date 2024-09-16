use crate::interfaces::contract::ERC20Event;
use crate::interfaces::event::{self as EventInterface, ERCCompliance};
use crate::services::contract::common::utils::parse_u256;
use starknet::providers::sequencer::models::Event;

pub fn decode(
    event: &Event,
) -> Result<Option<(ERC20Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.is_empty() {
        return Ok(None);
    }

    match event.keys[0] {
        key if key == EventInterface::TRANSFER => decode_transfer(event),
        key if key == EventInterface::APPROVAL => decode_approval(event),
        _ => Ok(None),
    }
}

fn decode_transfer(
    event: &Event,
) -> Result<Option<(ERC20Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 3 && event.data.len() == 2 {
        let value = parse_u256(&event.data[0], &event.data[1]);
        Ok(Some((
            ERC20Event::Transfer {
                from: event.keys[1],
                to: event.keys[2],
                value,
            },
            ERCCompliance::OPENZEPPELIN,
        )))
    } else if event.keys.len() == 1 && event.data.len() == 4 {
        let value = parse_u256(&event.data[2], &event.data[3]);
        Ok(Some((
            ERC20Event::Transfer {
                from: event.data[0],
                to: event.data[1],
                value,
            },
            ERCCompliance::OTHER,
        )))
    } else {
        Ok(None)
    }
}

fn decode_approval(
    event: &Event,
) -> Result<Option<(ERC20Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() == 3 && event.data.len() == 2 {
        let value = parse_u256(&event.data[0], &event.data[1]).to_string();
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

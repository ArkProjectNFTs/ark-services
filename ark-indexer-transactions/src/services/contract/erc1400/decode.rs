use crate::interfaces::contract::ERC1400Event;
use crate::interfaces::event::ERCCompliance;
use crate::services::contract::common::utils::parse_u256;
use starknet::providers::sequencer::models::Event;

pub fn decode(
    event: &Event,
) -> Result<Option<(ERC1400Event, ERCCompliance)>, Box<dyn std::error::Error + Send + Sync>> {
    if event.keys.len() >= 3 && event.data.len() >= 2 {
        let value = parse_u256(&event.data[0], &event.data[1]);
        Ok(Some((
            ERC1400Event::Transfer {
                from: event.keys[1],
                to: event.keys[2],
                value,
            },
            ERCCompliance::OTHER,
        )))
    } else {
        Ok(None)
    }
}

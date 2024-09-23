use bigdecimal::BigDecimal;
use num_traits::ToPrimitive;
use starknet::core::types::{Felt, U256};
use std::str::FromStr;

pub fn parse_u256(low: &Felt, high: &Felt) -> BigDecimal {
    let low = low.to_u128().unwrap_or(0);
    let high = high.to_u128().unwrap_or(0);
    BigDecimal::from_str(&U256::from_words(low, high).to_string()).unwrap_or_default()
}

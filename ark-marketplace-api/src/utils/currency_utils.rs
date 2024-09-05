use bigdecimal::BigDecimal;
use bigdecimal::Zero;
use std::str::FromStr;

pub fn compute_floor_difference(
    currency_amount: Option<BigDecimal>,
    _currency_address: String,
    floor_price: Option<BigDecimal>,
) -> Option<BigDecimal> {
    if currency_amount.is_none() || floor_price.is_none() {
        None
    } else {
        let floor_price = floor_price.unwrap();
        let price = currency_amount.unwrap(); // TODO: handle currency conversion
        let diff = price - floor_price.clone();
        if diff.is_zero() {
            return Some(BigDecimal::from(0));
        }
        if floor_price.is_zero() {
            None
        } else {
            let percentage_diff = (diff / floor_price) * BigDecimal::from(100);
            let percentage_diff_str = percentage_diff.to_string();
            let trimmed_str = percentage_diff_str
                .trim_end_matches('0')
                .trim_end_matches('.');
            let rounded_percentage_diff = BigDecimal::from_str(trimmed_str).unwrap();

            Some(rounded_percentage_diff)
        }
    }
}

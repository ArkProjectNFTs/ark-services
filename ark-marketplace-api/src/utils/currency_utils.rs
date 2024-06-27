use bigdecimal::BigDecimal;
use bigdecimal::Zero;

pub fn compute_floor_difference(
    currency_amount: Option<BigDecimal>,
    _currency_address: String,
    floor_price: Option<BigDecimal>,
) -> Option<BigDecimal> {
    if currency_amount.is_none() || floor_price.is_none() {
        None
    } else {
        let floor_price = floor_price.unwrap();
        let diff = currency_amount.unwrap() - floor_price.clone();
        if floor_price.is_zero() {
            None
        } else {
            Some(diff / floor_price)
        }
    }
}

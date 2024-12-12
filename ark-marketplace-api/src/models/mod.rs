pub mod collection;
pub mod default;
pub mod portfolio;
pub mod token;

fn serialize_option_bigdecimal<S>(
    value: &Option<bigdecimal::BigDecimal>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(v) => serializer.serialize_str(&v.to_plain_string()),
        None => serializer.serialize_str(""),
    }
}

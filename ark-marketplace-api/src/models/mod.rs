pub mod collection;
pub mod default;
pub mod portfolio;
pub mod token;

use std::str::FromStr;

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

fn deserialize_option_bigdecimal<'de, D>(
    deserializer: D,
) -> Result<Option<bigdecimal::BigDecimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        match bigdecimal::BigDecimal::from_str(&s) {
            Ok(val) => Ok(Some(val)),
            Err(e) => Err(serde::de::Error::custom(format!(
                "Failed to parse BigDecimal: {}",
                e
            ))),
        }
    }
}

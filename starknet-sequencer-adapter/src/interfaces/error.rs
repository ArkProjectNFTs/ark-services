use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ArkError(pub String);

impl fmt::Display for ArkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ArkError {}

use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serializer};

/// # Errors
pub fn serialize<S>(mime: &mime::Mime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(mime.to_string().as_str())
}

/// # Errors
pub fn deserialize<'de, D>(deserializer: D) -> Result<mime::Mime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(mime::Mime::from_str(s.as_str()).unwrap_or(mime::APPLICATION_OCTET_STREAM))
}

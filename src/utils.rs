use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize_mime<S>(mime: &mime::Mime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(mime.essence_str())
}

pub fn deserialize_mime<'de, D>(deserializer: D) -> Result<mime::Mime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mime = mime::Mime::from_str(s.as_str()).unwrap_or(mime::APPLICATION_OCTET_STREAM);
    Ok(mime)
}

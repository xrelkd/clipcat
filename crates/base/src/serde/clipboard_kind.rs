use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serializer};

use crate::ClipboardKind;

/// # Errors
pub fn serialize<S>(kind: &ClipboardKind, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(kind.to_string().as_str())
}

/// # Errors
pub fn deserialize<'de, D>(deserializer: D) -> Result<ClipboardKind, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(ClipboardKind::from_str(s.as_str()).unwrap_or_default())
}

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    time::SystemTime,
};

use crate::{utils, ClipboardEvent, ClipboardMode};

#[derive(Debug, Eq, Clone, Serialize, Deserialize)]
pub struct ClipEntry {
    pub id: u64,
    pub data: Vec<u8>,
    pub mode: ClipboardMode,
    pub timestamp: SystemTime,

    #[serde(
        serialize_with = "utils::serialize_mime",
        deserialize_with = "utils::deserialize_mime"
    )]
    pub mime: mime::Mime,
}

impl ClipEntry {
    #[inline]
    pub fn new(data: &[u8], mime: mime::Mime, clipboard_mode: ClipboardMode) -> ClipEntry {
        let id = Self::compute_id(data);
        ClipEntry {
            id,
            data: data.into(),
            mime,
            mode: clipboard_mode,
            timestamp: SystemTime::now(),
        }
    }

    #[inline]
    pub fn from_string<S: ToString>(s: S, clipboard_mode: ClipboardMode) -> ClipEntry {
        Self::new(s.to_string().as_bytes(), mime::TEXT_PLAIN_UTF_8, clipboard_mode)
    }

    #[inline]
    pub fn compute_id(data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut s = DefaultHasher::new();
        data.hash(&mut s);
        s.finish()
    }

    #[inline]
    pub fn is_text(&self) -> bool { self.mime.type_() == mime::TEXT }

    #[inline]
    pub fn is_utf8_string(&self) -> bool { self.mime.get_param(mime::CHARSET) == Some(mime::UTF_8) }

    #[inline]
    pub fn as_utf8_string(&self) -> String { String::from_utf8_lossy(&self.data).into() }

    pub fn printable_data(&self, line_length: Option<usize>) -> String {
        fn truncate(s: &str, max_chars: usize) -> &str {
            match s.char_indices().nth(max_chars) {
                None => s,
                Some((idx, _)) => &s[..idx],
            }
        }

        let data: String = {
            if self.is_utf8_string() || self.is_text() {
                self.as_utf8_string()
            } else {
                format!(
                    "type: {}, size: {}, time: {:?}",
                    self.mime.essence_str(),
                    self.data.len(),
                    self.timestamp
                )
            }
        };

        let data = match line_length {
            None | Some(0) => data,
            Some(limit) => {
                let char_count = data.chars().count();
                let line_count = data.lines().count();
                if char_count > limit {
                    let line_info = if line_count > 1 {
                        format!("...({} lines)", line_count)
                    } else {
                        "...".to_owned()
                    };
                    let mut data = truncate(&data, limit - line_info.len()).to_owned();
                    data.push_str(&line_info);
                    data
                } else {
                    data
                }
            }
        };

        data.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")
    }

    #[inline]
    pub fn mark(&mut self, clipboard_mode: ClipboardMode) {
        self.mode = clipboard_mode;
        self.timestamp = SystemTime::now();
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] { &self.data }

    #[inline]
    pub fn mime(&self) -> &mime::Mime { &self.mime }

    #[inline]
    pub fn mime_str(&self) -> &str { &self.mime.essence_str() }
}

impl From<ClipboardEvent> for ClipEntry {
    fn from(event: ClipboardEvent) -> ClipEntry {
        let ClipboardEvent { data, mime, mode } = event;
        let id = Self::compute_id(&data);
        let timestamp = SystemTime::now();
        ClipEntry { id, data, mime, mode, timestamp }
    }
}

impl Default for ClipEntry {
    fn default() -> ClipEntry {
        ClipEntry {
            id: 0,
            data: Default::default(),
            mime: mime::TEXT_PLAIN_UTF_8,
            mode: ClipboardMode::Selection,
            timestamp: SystemTime::UNIX_EPOCH,
        }
    }
}

impl PartialEq for ClipEntry {
    fn eq(&self, other: &Self) -> bool { self.data == other.data }
}

impl PartialOrd for ClipEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for ClipEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.timestamp.cmp(&self.timestamp) {
            Ordering::Equal => self.mode.cmp(&other.mode),
            ord => ord,
        }
    }
}

impl Hash for ClipEntry {
    fn hash<H: Hasher>(&self, state: &mut H) { self.data.hash(state); }
}

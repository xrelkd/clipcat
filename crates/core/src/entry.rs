use std::{
    cmp::Ordering,
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
    time::SystemTime,
};

use chrono::{offset::Utc, DateTime};
use image::ImageEncoder as _;
use snafu::{ResultExt, Snafu};

use crate::{ClipboardContent, ClipboardKind};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Eq)]
pub struct ClipEntry {
    id: u64,

    content: ClipboardContent,

    clipboard_kind: ClipboardKind,

    timestamp: SystemTime,
}

impl ClipEntry {
    /// # Errors
    #[inline]
    pub fn new(
        data: &[u8],
        mime: &mime::Mime,
        clipboard_kind: ClipboardKind,
        timestamp: Option<SystemTime>,
    ) -> Result<Self, Error> {
        let content = if mime.type_() == mime::TEXT {
            ClipboardContent::Plaintext(String::from_utf8_lossy(data).to_string())
        } else if mime.subtype() == mime::PNG {
            let cursor = std::io::Cursor::new(&data);
            let mut reader = image::io::Reader::new(cursor);
            reader.set_format(image::ImageFormat::Png);
            reader
                .decode()
                .map(|img| {
                    let image = img.into_rgba8();
                    let (w, h) = image.dimensions();
                    ClipboardContent::Image {
                        width: w as usize,
                        height: h as usize,
                        bytes: image.into_raw().into(),
                    }
                })
                .context(ConverseImageSnafu {})?
        } else {
            return Err(Error::FormatNotAvailable);
        };

        Ok(Self {
            id: Self::compute_id(&content),
            content,
            clipboard_kind,
            timestamp: timestamp.unwrap_or_else(SystemTime::now),
        })
    }

    #[inline]
    pub fn from_string<S: fmt::Display>(s: S, clipboard_kind: ClipboardKind) -> Self {
        Self::new(s.to_string().as_bytes(), &mime::TEXT_PLAIN_UTF_8, clipboard_kind, None)
            .unwrap_or_default()
    }

    #[inline]
    pub fn from_clipboard_content(
        content: ClipboardContent,
        clipboard_kind: ClipboardKind,
    ) -> Self {
        Self {
            id: Self::compute_id(&content),
            content,
            clipboard_kind,
            timestamp: SystemTime::now(),
        }
    }

    #[inline]
    #[must_use]
    pub fn compute_id(data: &ClipboardContent) -> u64 {
        let mut s = DefaultHasher::new();
        data.hash(&mut s);
        s.finish()
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> u64 { self.id }

    #[inline]
    #[must_use]
    pub const fn kind(&self) -> ClipboardKind { self.clipboard_kind }

    #[inline]
    #[must_use]
    pub const fn timestamp(&self) -> SystemTime { self.timestamp }

    #[inline]
    #[must_use]
    pub const fn is_utf8_string(&self) -> bool {
        matches!(self.content, ClipboardContent::Plaintext(_))
    }

    #[inline]
    #[must_use]
    pub fn as_utf8_string(&self) -> String {
        if let ClipboardContent::Plaintext(text) = &self.content {
            text.clone()
        } else {
            String::new()
        }
    }

    #[must_use]
    pub fn printable_data(&self, line_length: Option<usize>) -> String {
        fn truncate(s: &str, max_chars: usize) -> &str {
            match s.char_indices().nth(max_chars) {
                None => s,
                Some((idx, _)) => &s[..idx],
            }
        }

        let data = match &self.content {
            ClipboardContent::Plaintext(text) => text.clone(),
            ClipboardContent::Image { width: _, height: _, bytes } => {
                let content_type = mime::IMAGE_PNG;
                let size = bytes.len();
                let timestamp = DateTime::<Utc>::from(self.timestamp).to_rfc3339();
                format!("content-type: {content_type}, size: {size}, timestamp: {timestamp}")
            }
        };

        let data = match line_length {
            None | Some(0) => data,
            Some(limit) => {
                let char_count = data.chars().count();
                let line_count = data.lines().count();
                if char_count > limit {
                    let line_info = if line_count > 1 {
                        format!("...({line_count} lines)")
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

        data.replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
    }

    #[inline]
    pub fn mark(&mut self, clipboard_kind: ClipboardKind) {
        self.clipboard_kind = clipboard_kind;
        self.timestamp = SystemTime::now();
    }

    #[must_use]
    pub fn to_clipboard_content(&self) -> ClipboardContent { self.content.clone() }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool { self.content.is_empty() }

    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match &self.content {
            ClipboardContent::Plaintext(text) => text.as_bytes(),
            ClipboardContent::Image { bytes, .. } => bytes,
        }
    }

    /// # Errors
    #[inline]
    pub fn encoded(&self) -> Result<Vec<u8>, Error> {
        match &self.content {
            ClipboardContent::Plaintext(text) => Ok(text.as_bytes().to_vec()),
            ClipboardContent::Image { width, height, bytes } => {
                encode_as_png(*width, *height, bytes)
            }
        }
    }

    #[inline]
    #[must_use]
    pub const fn mime(&self) -> mime::Mime {
        match self.content {
            ClipboardContent::Plaintext(_) => mime::TEXT_PLAIN_UTF_8,
            ClipboardContent::Image { .. } => mime::IMAGE_PNG,
        }
    }
}

impl Default for ClipEntry {
    fn default() -> Self {
        Self {
            id: 0,
            content: ClipboardContent::Plaintext(String::new()),
            clipboard_kind: ClipboardKind::Clipboard,
            timestamp: SystemTime::UNIX_EPOCH,
        }
    }
}

impl PartialEq for ClipEntry {
    fn eq(&self, other: &Self) -> bool { self.content == other.content }
}

impl PartialOrd for ClipEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for ClipEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.timestamp.cmp(&self.timestamp) {
            Ordering::Equal => self.clipboard_kind.cmp(&other.clipboard_kind),
            ord => ord,
        }
    }
}

impl Hash for ClipEntry {
    fn hash<H: Hasher>(&self, state: &mut H) { self.content.hash(state); }
}

fn encode_as_png(width: usize, height: usize, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let (width, height) =
        (u32::try_from(width).unwrap_or_default(), u32::try_from(height).unwrap_or_default());
    if bytes.is_empty() || width == 0 || height == 0 {
        return Err(Error::EmptyImage);
    }

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(bytes.as_ref(), width, height, image::ColorType::Rgba8)
        .context(ConverseImageSnafu {})?;

    Ok(png_bytes)
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("The format is not available"))]
    FormatNotAvailable,

    #[snafu(display("The image is empty"))]
    EmptyImage,

    #[snafu(display("Error occurs while conversing image, error: {source}"))]
    ConverseImage { source: image::ImageError },
}

use std::{
    cmp::Ordering,
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use image::ImageEncoder as _;
use snafu::{ResultExt, Snafu};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

use crate::{ClipboardContent, ClipboardKind};

#[derive(Clone, Debug, Eq)]
pub struct Entry {
    id: u64,

    content: ClipboardContent,

    clipboard_kind: ClipboardKind,

    timestamp: OffsetDateTime,
}

impl Entry {
    /// # Errors
    #[inline]
    pub fn new(
        data: &[u8],
        mime: &mime::Mime,
        clipboard_kind: ClipboardKind,
        timestamp: Option<OffsetDateTime>,
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
                .context(ConvertImageSnafu {})?
        } else {
            return Err(Error::FormatNotAvailable);
        };

        Ok(Self {
            id: Self::compute_id(&content),
            content,
            clipboard_kind,
            timestamp: timestamp.unwrap_or_else(OffsetDateTime::now_utc),
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
        timestamp: Option<OffsetDateTime>,
    ) -> Self {
        Self {
            id: Self::compute_id(&content),
            content,
            clipboard_kind,
            timestamp: timestamp.unwrap_or_else(OffsetDateTime::now_utc),
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
    pub const fn timestamp(&self) -> OffsetDateTime { self.timestamp }

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
                let size = humansize::format_size(bytes.len(), humansize::BINARY);
                let timestamp = self
                    .timestamp
                    .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC))
                    .format(&Rfc3339)
                    .unwrap_or_default();
                format!("[{content_type} {size} {timestamp}]")
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
        self.timestamp = OffsetDateTime::now_utc();
    }

    #[must_use]
    pub fn to_clipboard_content(&self) -> ClipboardContent { self.content.clone() }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool { self.content.is_empty() }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize { self.content.len() }

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

    #[inline]
    pub fn metadata(&self, preview_length: Option<usize>) -> Metadata {
        Metadata {
            id: self.id,
            kind: self.clipboard_kind,
            timestamp: self.timestamp,
            mime: self.mime(),
            preview: self.printable_data(preview_length),
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            id: 0,
            content: ClipboardContent::Plaintext(String::new()),
            clipboard_kind: ClipboardKind::Clipboard,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool { self.content == other.content }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.timestamp.cmp(&self.timestamp) {
            Ordering::Equal => self.clipboard_kind.cmp(&other.clipboard_kind),
            ord => ord,
        }
    }
}

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) { self.content.hash(state); }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    pub id: u64,

    pub kind: ClipboardKind,

    pub timestamp: OffsetDateTime,

    pub mime: mime::Mime,

    pub preview: String,
}

impl PartialOrd for Metadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Metadata {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.timestamp.cmp(&self.timestamp) {
            Ordering::Equal => self.kind.cmp(&other.kind),
            ord => ord,
        }
    }
}

fn encode_as_png(width: usize, height: usize, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let (width, height) =
        (u32::try_from(width).unwrap_or_default(), u32::try_from(height).unwrap_or_default());
    if bytes.is_empty() || width == 0 || height == 0 {
        return Err(Error::EmptyImage);
    }

    let mut png_bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_bytes)
        .write_image(bytes, width, height, image::ColorType::Rgba8)
        .context(ConvertImageSnafu {})?;

    Ok(png_bytes)
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("The format is not available"))]
    FormatNotAvailable,

    #[snafu(display("The image is empty"))]
    EmptyImage,

    #[snafu(display("Error occurs while converting image, error: {source}"))]
    ConvertImage { source: image::ImageError },
}

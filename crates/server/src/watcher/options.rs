use std::collections::HashSet;

use clipcat_base::{ClipFilter, ClipboardKind};
use snafu::Snafu;

// SAFETY: user may use bool to enable/disable the functions
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct Options {
    pub load_current: bool,

    pub enable_clipboard: bool,

    pub enable_primary: bool,

    pub capture_image: bool,

    pub filter_text_min_length: usize,

    pub filter_text_max_length: usize,

    pub filter_image_max_size: usize,

    pub denied_text_regex_patterns: HashSet<String>,

    pub sensitive_x11_atoms: HashSet<String>,
}

impl Options {
    /// # Errors
    pub fn generate_clip_filter(&self) -> Result<ClipFilter, Error> {
        let mut filter = ClipFilter::new();
        filter.set_text_min_length(self.filter_text_min_length);
        filter.set_text_max_length(self.filter_text_max_length);
        filter.set_image_max_size(self.filter_image_max_size);
        filter.deny_image(!self.capture_image);
        filter.set_regex_patterns(regex::RegexSet::new(&self.denied_text_regex_patterns)?);
        filter.add_sensitive_atoms(self.sensitive_x11_atoms.clone());
        Ok(filter)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            load_current: true,
            enable_clipboard: true,
            enable_primary: true,
            capture_image: true,
            filter_text_min_length: 1,
            filter_text_max_length: 5 * (1 << 20),
            // 5 MiB
            filter_image_max_size: 5 * (1 << 20),
            denied_text_regex_patterns: HashSet::new(),
            sensitive_x11_atoms: HashSet::new(),
        }
    }
}

impl Options {
    #[inline]
    pub(crate) fn get_enable_kinds(&self) -> [bool; ClipboardKind::MAX_LENGTH] {
        let mut kinds = [false; ClipboardKind::MAX_LENGTH];
        if self.enable_clipboard {
            kinds[usize::from(ClipboardKind::Clipboard)] = true;
        }
        if self.enable_primary {
            kinds[usize::from(ClipboardKind::Primary)] = true;
        }
        if kinds.iter().all(|x| !x) {
            tracing::warn!("Both clipboard and primary are not watched");
        }
        kinds
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to parse regular expression, error: {error}"))]
    ParseRegularExpressions { error: regex::Error },
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self { Self::ParseRegularExpressions { error } }
}

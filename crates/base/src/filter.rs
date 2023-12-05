use std::collections::HashSet;

use crate::ClipboardContent;

#[derive(Clone, Debug)]
pub struct Filter {
    regex_set: regex::RegexSet,
    sensitive_atoms: HashSet<String>,
    deny_image: bool,
    filter_text_min_length: usize,
    filter_text_max_length: usize,
    filter_image_max_size: usize,
}

impl Filter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            regex_set: regex::RegexSet::default(),

            sensitive_atoms: HashSet::new(),

            deny_image: false,

            filter_text_min_length: 1,

            // 2,000,000 characters
            filter_text_max_length: 2_000_000,

            // 5 MiB
            filter_image_max_size: 5 * (1 << 20),
        }
    }

    pub fn add_sensitive_atoms<I>(&mut self, sensitive_atoms: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.sensitive_atoms.extend(sensitive_atoms);
    }

    pub fn set_regex_patterns(&mut self, regex_patterns: regex::RegexSet) {
        self.regex_set = regex_patterns;
    }

    pub fn set_text_min_length(&mut self, size: usize) { self.filter_text_min_length = size; }

    pub fn set_text_max_length(&mut self, size: usize) { self.filter_text_max_length = size; }

    pub fn set_image_max_size(&mut self, size: usize) { self.filter_image_max_size = size; }

    pub fn deny_image(&mut self, deny_image: bool) { self.deny_image = deny_image; }

    pub fn filter_clipboard_content<C>(&self, content: C) -> bool
    where
        C: AsRef<ClipboardContent>,
    {
        match content.as_ref() {
            ClipboardContent::Plaintext(text) => {
                self.filter_by_text_size(text) || self.filter_text_by_regular_expression(text)
            }
            ClipboardContent::Image { bytes, .. } => {
                self.deny_image || self.filter_by_image_size(bytes)
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn filter_sensitive_atoms<'a, I>(&self, mut atoms: I) -> bool
    where
        I: Iterator<Item = &'a String>,
    {
        atoms.any(|atom| self.sensitive_atoms.contains(atom))
    }

    #[inline]
    #[must_use]
    pub fn filter_by_mime_type(&self, mime: &mime::Mime) -> bool {
        self.deny_image && mime.type_() == mime::IMAGE
    }

    #[inline]
    #[must_use]
    pub fn filter_by_text_size<S>(&self, text: S) -> bool
    where
        S: AsRef<str>,
    {
        let count = text.as_ref().chars().count();
        count <= self.filter_text_min_length || count > self.filter_text_max_length
    }

    #[inline]
    pub fn filter_text_by_regular_expression<S>(&self, text: S) -> bool
    where
        S: AsRef<str>,
    {
        if self.regex_set.is_empty() {
            false
        } else {
            self.regex_set.is_match(text.as_ref())
        }
    }

    #[inline]
    #[must_use]
    pub fn filter_by_image_size<D>(&self, data: D) -> bool
    where
        D: AsRef<[u8]>,
    {
        data.as_ref().len() > self.filter_image_max_size
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            regex_set: regex::RegexSet::default(),

            sensitive_atoms: HashSet::from(["x-kde-passwordManagerHint".to_string()]),

            deny_image: false,

            filter_text_min_length: 1,
            // 5 MiB
            filter_text_max_length: 5 * (1 << 20),
            // 5 MiB
            filter_image_max_size: 5 * (1 << 20),
        }
    }
}

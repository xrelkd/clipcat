use clipcat_base::{ClipboardContent, ClipboardKind};

// SAFETY: user may use bool to enable/disable the functions
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub load_current: bool,

    pub enable_clipboard: bool,

    pub enable_primary: bool,

    pub capture_image: bool,

    pub filter_min_size: usize,

    pub filter_max_size: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            load_current: true,

            enable_clipboard: true,

            enable_primary: true,

            capture_image: true,

            filter_min_size: 1,
            // 5 MiB
            filter_max_size: 5 * (1 << 20),
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

    pub(crate) fn generate_content_checker(&self) -> impl Fn(&ClipboardContent) -> bool {
        let Self { capture_image, filter_max_size, filter_min_size, .. } = *self;
        move |data: &ClipboardContent| -> bool {
            let ret = (data.is_plaintext() || (capture_image && data.is_image()))
                && data.len() > filter_min_size
                && data.len() <= filter_max_size;
            if !ret {
                tracing::info!(
                    "Clip ({info}) is ignored, because of the configuration (filter_min_size: \
                     {filter_min_size}, filter_max_size: {filter_max_size}, capture_image: \
                     {capture_image})",
                    info = data.basic_information()
                );
            }
            ret
        }
    }
}

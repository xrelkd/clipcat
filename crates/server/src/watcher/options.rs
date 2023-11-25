#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub load_current: bool,

    pub enable_clipboard: bool,

    pub enable_primary: bool,

    pub filter_min_size: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            load_current: true,
            enable_clipboard: true,
            enable_primary: true,
            filter_min_size: 1,
        }
    }
}

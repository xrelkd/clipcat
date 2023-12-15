use clipcat_clipboard::{Error, LocalClipboard};

mod common;

use self::common::ClipboardTester;

#[derive(Debug)]
pub struct LocalClipboardTester;

impl Default for LocalClipboardTester {
    fn default() -> Self { Self::new() }
}

impl LocalClipboardTester {
    #[must_use]
    pub const fn new() -> Self { Self }
}

impl ClipboardTester for LocalClipboardTester {
    type Clipboard = LocalClipboard;

    fn new_clipboard(&self) -> Result<Self::Clipboard, Error> { Ok(LocalClipboard::new()) }
}

#[test]
fn test_local() -> Result<(), Error> { LocalClipboardTester::new().run() }

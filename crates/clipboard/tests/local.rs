use clipcat_clipboard::{Error, LocalClipboard};

mod common;

use self::common::ClipboardTester;

#[derive(Debug)]
pub struct Tester;

impl Default for Tester {
    fn default() -> Self { Self::new() }
}

impl Tester {
    #[must_use]
    pub const fn new() -> Self { Self }
}

impl ClipboardTester for Tester {
    type Clipboard = LocalClipboard;

    fn new_clipboard(&self) -> Result<Self::Clipboard, Error> { Ok(LocalClipboard::new()) }
}

#[test]
fn test_local() -> Result<(), Error> { Tester::new().run() }

use clipcat_clipboard::{Error, MockClipboard};

mod common;

use self::common::ClipboardTester;

#[derive(Debug)]
pub struct MockClipboardTester;

impl Default for MockClipboardTester {
    fn default() -> Self { Self::new() }
}

impl MockClipboardTester {
    #[must_use]
    pub const fn new() -> Self { Self }
}

impl ClipboardTester for MockClipboardTester {
    type Clipboard = MockClipboard;

    fn new_clipboard(&self) -> Result<Self::Clipboard, Error> { Ok(MockClipboard::new()) }
}

#[test]
fn test_mock() -> Result<(), Error> { MockClipboardTester::new().run() }

use clipcat_clipboard::{Clipboard, ClipboardKind, Error};

mod common;

use self::common::ClipboardTester;

#[derive(Debug)]
pub struct DefaultClipboardTester {
    kind: ClipboardKind,
}

impl DefaultClipboardTester {
    #[must_use]
    pub const fn new(kind: ClipboardKind) -> Self { Self { kind } }
}

impl ClipboardTester for DefaultClipboardTester {
    type Clipboard = Clipboard;

    fn new_clipboard(&self) -> Self::Clipboard { Clipboard::new(None, self.kind).unwrap() }
}

#[test]
fn test_clipboard() -> Result<(), Error> {
    let tester = DefaultClipboardTester::new(ClipboardKind::Clipboard);
    tester.run()
}

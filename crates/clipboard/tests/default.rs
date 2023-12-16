use std::sync::Arc;

use clipcat_clipboard::{Clipboard, ClipboardKind, Error, X11ListenerError};

mod common;

use self::common::ClipboardTester;

#[derive(Debug)]
pub struct Tester {
    kind: ClipboardKind,
}

impl Tester {
    #[must_use]
    pub const fn new(kind: ClipboardKind) -> Self { Self { kind } }
}

impl ClipboardTester for Tester {
    type Clipboard = Clipboard;

    fn new_clipboard(&self) -> Result<Self::Clipboard, Error> {
        let clipboard = Clipboard::new(self.kind, Arc::default(), Vec::new())?;
        Ok(clipboard)
    }
}

#[test]
fn test_x11_clipboard() -> Result<(), Error> {
    match Tester::new(ClipboardKind::Clipboard).run() {
        Err(Error::X11Listener { error: X11ListenerError::Connect { .. } }) => {
            eprintln!("Could not connect to X11 server, skip the further test cases");
            Ok(())
        }
        result => result,
    }
}

#[test]
fn test_x11_primary() -> Result<(), Error> {
    match Tester::new(ClipboardKind::Primary).run() {
        Err(Error::X11Listener { error: X11ListenerError::Connect { .. } }) => {
            eprintln!("Could not connect to X11 server, skip the further test cases");
            Ok(())
        }
        result => result,
    }
}

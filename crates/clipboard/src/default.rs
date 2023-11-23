use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use arboard::{ClearExtLinux, GetExtLinux, SetExtLinux};
use bytes::Bytes;
use clipcat::ClipboardContent;

use crate::{
    listener::Listener, ClipboardKind, ClipboardLoad, ClipboardStore, ClipboardSubscribe, Error,
    Subscriber,
};

#[derive(Clone)]
pub struct Clipboard {
    listener: Arc<Listener>,
    clipboard_kind: arboard::LinuxClipboardKind,
    clear_on_drop: Arc<AtomicBool>,
}

impl Clipboard {
    /// # Errors
    pub fn new(display_name: Option<String>, clipboard_kind: ClipboardKind) -> Result<Self, Error> {
        let listener = Arc::new(Listener::new(display_name, clipboard_kind)?);
        let clear_on_drop = Arc::new(AtomicBool::from(false));
        let clipboard_kind = match clipboard_kind {
            ClipboardKind::Clipboard => arboard::LinuxClipboardKind::Clipboard,
            ClipboardKind::Primary => arboard::LinuxClipboardKind::Primary,
            ClipboardKind::Secondary => arboard::LinuxClipboardKind::Secondary,
        };
        Ok(Self { listener, clipboard_kind, clear_on_drop })
    }
}

impl ClipboardSubscribe for Clipboard {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Self::Subscriber, Error> { self.listener.subscribe() }
}

impl ClipboardLoad for Clipboard {
    fn load(&self) -> Result<ClipboardContent, Error> {
        let mut arboard = arboard::Clipboard::new()?;

        match arboard.get().clipboard(self.clipboard_kind).text() {
            Ok(text) => Ok(ClipboardContent::Plaintext(text)),
            Err(arboard::Error::ContentNotAvailable) => {
                match arboard.get().clipboard(self.clipboard_kind).image() {
                    Ok(arboard::ImageData { width, height, bytes }) => {
                        Ok(ClipboardContent::Image {
                            width,
                            height,
                            bytes: Bytes::from(bytes.into_owned()),
                        })
                    }
                    Err(arboard::Error::ClipboardNotSupported) => unreachable!(),
                    Err(_err) => Err(Error::Empty),
                }
            }
            Err(arboard::Error::ClipboardNotSupported) => unreachable!(),
            Err(_err) => Err(Error::Empty),
        }
    }
}

impl ClipboardStore for Clipboard {
    #[inline]
    fn store(&self, content: ClipboardContent) -> Result<(), Error> {
        let mut arboard = arboard::Clipboard::new()?;
        let clipboard_kind = self.clipboard_kind;
        let clear_on_drop = self.clear_on_drop.clone();

        let _join_handle = std::thread::spawn(move || {
            clear_on_drop.store(true, Ordering::Relaxed);

            let _result = match content {
                ClipboardContent::Plaintext(text) => {
                    arboard.set().clipboard(clipboard_kind).wait().text(text)
                }
                ClipboardContent::Image { width, height, bytes } => arboard
                    .set()
                    .clipboard(clipboard_kind)
                    .wait()
                    .image(arboard::ImageData { width, height, bytes: bytes.to_vec().into() }),
            };

            clear_on_drop.store(false, Ordering::Relaxed);
        });
        Ok(())
    }

    #[inline]
    fn clear(&self) -> Result<(), Error> {
        arboard::Clipboard::new()?.clear_with().clipboard(self.clipboard_kind)?;
        self.clear_on_drop.store(false, Ordering::Relaxed);
        Ok(())
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        if self.clear_on_drop.load(Ordering::Relaxed) {
            drop(self.clear());
        }
    }
}

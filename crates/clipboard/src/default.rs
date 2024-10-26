use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
use arboard::{ClearExtLinux, GetExtLinux, SetExtLinux};
use bytes::Bytes;
use clipcat_base::{ClipFilter, ClipboardContent};

#[cfg(target_os = "macos")]
use crate::listener::MacOsListener;
#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
use crate::listener::X11Listener;
use crate::{
    traits::EventObserver, ClipboardKind, ClipboardLoad, ClipboardStore, ClipboardSubscribe, Error,
    Subscriber,
};

#[derive(Clone)]
pub struct Clipboard {
    listener: Arc<dyn ClipboardSubscribe<Subscriber = Subscriber>>,

    clear_on_drop: Arc<AtomicBool>,

    #[cfg(all(
        unix,
        not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "emscripten"
        ))
    ))]
    clipboard_kind: arboard::LinuxClipboardKind,
}

impl Clipboard {
    /// # Errors
    pub fn new(
        clipboard_kind: ClipboardKind,
        clip_filter: Arc<ClipFilter>,
        event_observers: Vec<Arc<dyn EventObserver>>,
    ) -> Result<Self, Error> {
        #[cfg(all(
            unix,
            not(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "android",
                target_os = "emscripten"
            ))
        ))]
        {
            Self::new_on_linux(clipboard_kind, clip_filter, event_observers)
        }

        #[cfg(target_os = "macos")]
        {
            let _ = clipboard_kind;
            drop(clip_filter);
            drop(event_observers);
            Self::new_on_macos()
        }
    }

    #[cfg(all(
        unix,
        not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "emscripten"
        ))
    ))]
    /// # Errors
    fn new_on_linux(
        clipboard_kind: ClipboardKind,
        clip_filter: Arc<ClipFilter>,
        event_observers: Vec<Arc<dyn EventObserver>>,
    ) -> Result<Self, Error> {
        let listener: Arc<dyn ClipboardSubscribe<Subscriber = Subscriber>> = {
            match std::env::var("DISPLAY") {
                Ok(display_name) => {
                    tracing::info!(
                        "Build X11 listener ({clipboard_kind}) with display `{display_name}`"
                    );
                    Arc::new(X11Listener::new(
                        Some(display_name),
                        clipboard_kind,
                        clip_filter,
                        event_observers,
                    )?)
                }
                Err(_) => {
                    Arc::new(X11Listener::new(None, clipboard_kind, clip_filter, event_observers)?)
                }
            }
        };

        let clear_on_drop = Arc::new(AtomicBool::from(false));

        let clipboard_kind = match clipboard_kind {
            ClipboardKind::Clipboard => arboard::LinuxClipboardKind::Clipboard,
            ClipboardKind::Primary => arboard::LinuxClipboardKind::Primary,
            ClipboardKind::Secondary => arboard::LinuxClipboardKind::Secondary,
        };
        Ok(Self { listener, clear_on_drop, clipboard_kind })
    }

    /// # Errors
    #[cfg(target_os = "macos")]
    pub fn new_on_macos() -> Result<Self, Error> {
        let listener: Arc<dyn ClipboardSubscribe<Subscriber = Subscriber>> =
            Arc::new(MacOsListener::new()?);

        let clear_on_drop = Arc::new(AtomicBool::from(false));

        Ok(Self { listener, clear_on_drop })
    }
}

impl ClipboardSubscribe for Clipboard {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Self::Subscriber, Error> { self.listener.subscribe() }
}

impl ClipboardLoad for Clipboard {
    fn load(&self, mime: Option<mime::Mime>) -> Result<ClipboardContent, Error> {
        match mime {
            None => self
                .load(Some(mime::TEXT_PLAIN_UTF_8))
                .map_or_else(|_| self.load(Some(mime::IMAGE_PNG)), Ok),
            Some(mime) => {
                let mut arboard = arboard::Clipboard::new()?;

                if mime.type_() == mime::TEXT {
                    #[cfg(all(
                        unix,
                        not(any(
                            target_os = "macos",
                            target_os = "ios",
                            target_os = "android",
                            target_os = "emscripten"
                        ))
                    ))]
                    let maybe_text = arboard.get().clipboard(self.clipboard_kind).text();

                    #[cfg(target_os = "macos")]
                    let maybe_text = arboard.get().text();

                    match maybe_text {
                        Ok(text) => Ok(ClipboardContent::Plaintext(text)),
                        Err(arboard::Error::ClipboardNotSupported) => unreachable!(),
                        Err(err) => {
                            tracing::warn!("{err}");
                            Err(Error::Empty)
                        }
                    }
                } else if mime.type_() == mime::IMAGE {
                    #[cfg(all(
                        unix,
                        not(any(
                            target_os = "macos",
                            target_os = "ios",
                            target_os = "android",
                            target_os = "emscripten"
                        ))
                    ))]
                    let maybe_image = arboard.get().clipboard(self.clipboard_kind).image();

                    #[cfg(target_os = "macos")]
                    let maybe_image = arboard.get().image();

                    match maybe_image {
                        Ok(arboard::ImageData { width, height, bytes }) => {
                            Ok(ClipboardContent::Image {
                                width,
                                height,
                                bytes: Bytes::from(bytes.into_owned()),
                            })
                        }
                        Err(arboard::Error::ClipboardNotSupported) => unreachable!(),
                        Err(err) => {
                            tracing::warn!("{err}");
                            Err(Error::Empty)
                        }
                    }
                } else {
                    Err(Error::Empty)
                }
            }
        }
    }
}

impl ClipboardStore for Clipboard {
    #[inline]
    fn store(&self, content: ClipboardContent) -> Result<(), Error> {
        let mut arboard = arboard::Clipboard::new()?;
        #[cfg(all(
            unix,
            not(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "android",
                target_os = "emscripten"
            ))
        ))]
        let clipboard_kind = self.clipboard_kind;

        #[cfg(target_os = "macos")]
        let clipboard_kind = ClipboardKind::Clipboard;

        let clear_on_drop = self.clear_on_drop.clone();

        let _join_handle =
            thread::Builder::new().name(format!("{clipboard_kind:?}-setter")).spawn(move || {
                clear_on_drop.store(true, Ordering::Relaxed);

                #[cfg(all(
                    unix,
                    not(any(
                        target_os = "macos",
                        target_os = "ios",
                        target_os = "android",
                        target_os = "emscripten"
                    ))
                ))]
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

                #[cfg(target_os = "macos")]
                let _result = match content {
                    ClipboardContent::Plaintext(text) => arboard.set().text(text),
                    ClipboardContent::Image { width, height, bytes } => arboard
                        .set()
                        .image(arboard::ImageData { width, height, bytes: bytes.to_vec().into() }),
                };

                clear_on_drop.store(false, Ordering::Relaxed);
            });
        Ok(())
    }

    #[inline]
    fn clear(&self) -> Result<(), Error> {
        #[cfg(all(
            unix,
            not(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "android",
                target_os = "emscripten"
            ))
        ))]
        arboard::Clipboard::new()?.clear_with().clipboard(self.clipboard_kind)?;

        #[cfg(target_os = "macos")]
        arboard::Clipboard::new()?.clear()?;

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

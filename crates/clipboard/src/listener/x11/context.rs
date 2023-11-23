use std::os::{fd::AsRawFd, unix::prelude::RawFd};

use snafu::ResultExt;
use x11rb::{
    connection::Connection,
    protocol::{xfixes, xfixes::ConnectionExt as _, xproto, xproto::ConnectionExt as _},
    rust_connection::RustConnection,
};

use crate::{
    listener::x11::{error, Error},
    ClipboardKind,
};

#[derive(Debug)]
pub struct Context {
    display_name: Option<String>,
    connection: RustConnection,
    window: xproto::Window,
    atom_cache: AtomCache,
    clipboard_kind: ClipboardKind,
}

impl Context {
    pub fn new(display_name: Option<String>, clipboard_kind: ClipboardKind) -> Result<Self, Error> {
        let (connection, window) = new_connection(display_name.as_deref())?;
        let atom_cache = AtomCache::new(&connection)?;
        let ctx = Self { display_name, connection, window, atom_cache, clipboard_kind };
        ctx.prepare_for_monitoring_event()?;
        Ok(ctx)
    }

    pub fn reconnect(&mut self) -> Result<(), Error> {
        let (connection, window) = new_connection(self.display_name.as_deref())?;
        self.atom_cache = AtomCache::new(&connection)?;
        self.connection = connection;
        self.window = window;
        self.prepare_for_monitoring_event()
    }

    #[inline]
    pub fn poll_for_event(&self) -> Result<x11rb::protocol::Event, Error> {
        self.connection.poll_for_event().context(error::PollForEventSnafu)?.ok_or(Error::NoEvent)
    }

    #[inline]
    const fn clipboard_kind(&self) -> xproto::Atom {
        match self.clipboard_kind {
            ClipboardKind::Clipboard => self.atom_cache.clipboard_selection,
            ClipboardKind::Primary => self.atom_cache.primary_selection,
            ClipboardKind::Secondary => self.atom_cache.secondary_selection,
        }
    }

    #[inline]
    fn flush(&self) -> Result<(), Error> {
        self.connection.flush().context(error::FlushConnectionSnafu)?;
        Ok(())
    }

    fn prepare_for_monitoring_event(&self) -> Result<(), Error> {
        const EXT_NAME: &str = "XFIXES";
        let xfixes = self
            .connection
            .query_extension(EXT_NAME.as_bytes())
            .with_context(|_| error::QueryExtensionSnafu { extension_name: EXT_NAME.to_string() })?
            .reply()
            .context(error::ReplySnafu)?;

        if !xfixes.present {
            return Err(error::Error::XfixesNotPresent);
        }

        {
            drop(
                self.connection
                    .xfixes_query_version(5, 0)
                    .context(error::QueryXfixesVersionSnafu)?,
            );

            drop(
                self.connection
                    .xfixes_select_selection_input(
                        self.window,
                        self.clipboard_kind(),
                        xproto::EventMask::NO_EVENT,
                    )
                    .context(error::SelectXfixesSelectionInputSnafu)?,
            );

            drop(
                self.connection
                    .xfixes_select_selection_input(
                        self.window,
                        self.clipboard_kind(),
                        xfixes::SelectionEventMask::SET_SELECTION_OWNER
                            | xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY
                            | xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE,
                    )
                    .context(error::SelectXfixesSelectionInputSnafu)?,
            );
        }

        self.flush()?;
        Ok(())
    }
}

impl AsRawFd for Context {
    fn as_raw_fd(&self) -> RawFd { self.connection.stream().as_raw_fd() }
}

#[derive(Debug)]
struct AtomCache {
    clipboard_selection: xproto::Atom,
    primary_selection: xproto::Atom,
    secondary_selection: xproto::Atom,
}

impl AtomCache {
    fn new(conn: &impl Connection) -> Result<Self, Error> {
        Ok(Self {
            clipboard_selection: get_intern_atom(conn, "CLIPBOARD")?,
            primary_selection: xproto::AtomEnum::PRIMARY.into(),
            secondary_selection: xproto::AtomEnum::SECONDARY.into(),
        })
    }
}

#[inline]
pub fn get_intern_atom(conn: &impl Connection, atom_name: &str) -> Result<xproto::Atom, Error> {
    conn.intern_atom(false, atom_name.as_bytes())
        .with_context(|_| error::GetAtomIdentifierByNameSnafu { atom_name: atom_name.to_string() })?
        .reply()
        .map(|r| r.atom)
        .context(error::ReplySnafu)
}

fn new_connection(display_name: Option<&str>) -> Result<(RustConnection, xproto::Window), Error> {
    let (connection, screen_num) =
        RustConnection::connect(display_name).context(error::ConnectSnafu)?;

    let window = {
        let window = connection.generate_id().context(error::GenerateX11IdentifierSnafu)?;
        let screen = &connection.setup().roots[screen_num];

        drop(
            connection
                .create_window(
                    x11rb::COPY_DEPTH_FROM_PARENT,
                    window,
                    screen.root,
                    0,
                    0,
                    1,
                    1,
                    0,
                    xproto::WindowClass::INPUT_OUTPUT,
                    screen.root_visual,
                    &xproto::CreateWindowAux::default().event_mask(
                        xproto::EventMask::PROPERTY_CHANGE, // | EventMask::STRUCTURE_NOTIFY
                    ),
                )
                .context(error::CreateWindowSnafu)?,
        );

        window
    };

    Ok((connection, window))
}

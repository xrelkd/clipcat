use std::sync::Arc;

use snafu::ResultExt;
use x11rb::{
    connection::Connection,
    protocol::{xfixes, xproto, xproto::ConnectionExt},
    rust_connection::RustConnection,
};

use crate::{
    listener::x11::{error, Error},
    ClipboardKind,
};

#[derive(Debug)]
pub struct Context {
    connection: RustConnection,
    window: u32,
    atom_cache: Arc<AtomCache>,
    clipboard_kind: ClipboardKind,
}

impl Context {
    #[inline]
    pub fn new(display_name: Option<&str>, clipboard_kind: ClipboardKind) -> Result<Self, Error> {
        let (connection, window) = Self::new_connection(display_name)?;
        let atom_cache = Arc::new(AtomCache::new(&connection)?);
        Ok(Self { connection, window, atom_cache, clipboard_kind })
    }

    fn new_connection(
        display_name: Option<&str>,
    ) -> Result<(RustConnection, xproto::Window), Error> {
        let (connection, screen_num) =
            RustConnection::connect(display_name).context(error::ConnectSnafu)?;

        let window =
            {
                let window = connection.generate_id().context(error::GenerateX11IdentifierSnafu)?;
                let screen = &connection.setup().roots[screen_num];

                drop(connection
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
                .context(error::CreateWindowSnafu)?);

                window
            };

        Ok((connection, window))
    }

    #[inline]
    pub fn clipboard_type(&self) -> xproto::Atom {
        match self.clipboard_kind {
            ClipboardKind::Clipboard => self.atom_cache.clipboard_selection,
            ClipboardKind::Primary => self.atom_cache.primary_selection,
            ClipboardKind::Secondary => self.atom_cache.secondary_selection,
        }
    }

    #[inline]
    pub fn close_event(&self) -> xproto::ClientMessageEvent {
        const CLOSE_CONNECTION_ATOM: xproto::Atom = 8293;
        xproto::ClientMessageEvent {
            response_type: xproto::CLIENT_MESSAGE_EVENT,
            sequence: 0,
            format: 32,
            window: self.window,
            type_: CLOSE_CONNECTION_ATOM,
            data: [0u32; 5].into(),
        }
    }

    #[inline]
    pub fn is_close_event(&self, event: &xproto::ClientMessageEvent) -> bool {
        let close_event = self.close_event();
        close_event.response_type == event.response_type
            && close_event.format == event.format
            && close_event.sequence == event.sequence
            && close_event.window == event.window
            && close_event.type_ == event.type_
    }

    #[inline]
    pub fn send_close_connection_event(&self) -> Result<(), Error> {
        let close_event = self.close_event();
        drop(
            self.connection
                .send_event(false, self.window, x11rb::NONE, close_event)
                .context(error::SendEventSnafu)?,
        );
        self.flush_connection()?;
        Ok(())
    }

    #[inline]
    pub fn flush_connection(&self) -> Result<(), Error> {
        self.connection.flush().context(error::FlushConnectionSnafu)?;
        Ok(())
    }

    #[inline]
    pub fn wait_for_event(&self) -> Result<x11rb::protocol::Event, Error> {
        self.connection.wait_for_event().context(error::WaitForEventSnafu)
    }

    pub fn prepare_for_monitoring_event(&self) -> Result<(), Error> {
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
            use xfixes::{ConnectionExt, SelectionEventMask};

            drop(
                self.connection
                    .xfixes_query_version(5, 0)
                    .context(error::QueryXfixesVersionSnafu)?,
            );

            drop(
                self.connection
                    .xfixes_select_selection_input(
                        self.window,
                        self.clipboard_type(),
                        xproto::EventMask::NO_EVENT,
                    )
                    .context(error::SelectXfixesSelectionInputSnafu)?,
            );

            drop(
                self.connection
                    .xfixes_select_selection_input(
                        self.window,
                        self.clipboard_type(),
                        SelectionEventMask::SET_SELECTION_OWNER
                            | SelectionEventMask::SELECTION_WINDOW_DESTROY
                            | SelectionEventMask::SELECTION_CLIENT_CLOSE,
                    )
                    .context(error::SelectXfixesSelectionInputSnafu)?,
            );
        }

        self.flush_connection()?;
        Ok(())
    }
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

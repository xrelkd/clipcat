use std::{
    os::{fd::AsRawFd, unix::prelude::RawFd},
    thread,
    time::{Duration, Instant},
};

use snafu::ResultExt;
use x11rb::{
    connection::Connection,
    protocol::{
        xfixes,
        xfixes::ConnectionExt as _,
        xproto::{self, ConnectionExt as _},
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

use crate::{
    listener::x11::{error, Error},
    ClipboardKind,
};

const LONG_TIMEOUT_DUR: Duration = Duration::from_millis(1000);

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
        ctx.prepare_for_listening_event()?;
        Ok(ctx)
    }

    pub fn reconnect(&mut self) -> Result<(), Error> {
        let (connection, window) = new_connection(self.display_name.as_deref())?;
        self.atom_cache = AtomCache::new(&connection)?;
        self.connection = connection;
        self.window = window;
        self.prepare_for_listening_event()
    }

    #[inline]
    pub fn poll_for_event(&self) -> Result<x11rb::protocol::Event, Error> {
        self.connection.poll_for_event().context(error::PollForEventSnafu)?.ok_or(Error::NoEvent)
    }

    #[inline]
    pub const fn clipboard_kind(&self) -> ClipboardKind { self.clipboard_kind }

    #[inline]
    const fn clipboard_kind_atom(&self) -> xproto::Atom {
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

    fn prepare_for_listening_event(&self) -> Result<(), Error> {
        const EXT_NAME: &str = "XFIXES";
        let xfixes = self
            .connection
            .query_extension(EXT_NAME.as_bytes())
            .with_context(|_| error::QueryExtensionSnafu { extension_name: EXT_NAME.to_string() })?
            .reply()
            .context(error::ReplySnafu)?;

        if !xfixes.present {
            return Err(Error::XfixesNotPresent);
        }

        drop(self.connection.set_selection_owner(
            self.window,
            self.atom_cache.clipboard_manager,
            x11rb::CURRENT_TIME,
        ));

        drop(self.connection.xfixes_query_version(5, 0).context(error::QueryXfixesVersionSnafu)?);

        drop(
            self.connection
                .xfixes_select_selection_input(
                    self.window,
                    self.clipboard_kind_atom(),
                    xfixes::SelectionEventMask::SET_SELECTION_OWNER
                        | xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY
                        | xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE,
                )
                .context(error::SelectXfixesSelectionInputSnafu)?,
        );

        self.flush()?;
        Ok(())
    }

    pub fn get_available_formats(&self) -> Result<Vec<String>, Error> {
        drop(
            self.connection
                .delete_property(self.window, self.atom_cache.clipcat_clipboard)
                .context(error::DeletePropertySnafu)?,
        );

        drop(
            self.connection
                .convert_selection(
                    self.window,
                    self.clipboard_kind_atom(),
                    self.atom_cache.targets,
                    self.atom_cache.clipcat_clipboard,
                    xproto::Time::CURRENT_TIME,
                )
                .context(error::ConvertSelectionSnafu)?,
        );
        self.connection.sync().context(error::SynchroniseWithX11Snafu)?;

        let timeout_end = Instant::now() + LONG_TIMEOUT_DUR;
        while Instant::now() < timeout_end {
            let maybe_event = self.connection.poll_for_event().context(error::PollForEventSnafu)?;
            let Some(event) = maybe_event else {
                thread::sleep(Duration::from_millis(1));
                continue;
            };

            match event {
                // The first response after requesting a selection.
                x11rb::protocol::Event::SelectionNotify(event) => {
                    let reply = self
                        .connection
                        .get_property(
                            false,
                            self.window,
                            event.property,
                            xproto::AtomEnum::NONE,
                            0,
                            u32::try_from(1024 * std::mem::size_of::<xproto::Atom>())
                                .unwrap_or(u32::MAX),
                        )
                        .context(error::GetPropertySnafu)?
                        .reply()
                        .context(error::GetPropertyReplySnafu)?;

                    let mut formats = Vec::new();
                    if let Some(atoms) = reply.value32() {
                        for atom in atoms {
                            let atom_name = self
                                .connection
                                .get_atom_name(atom)
                                .context(error::GetAtomNameSnafu)?
                                .reply()
                                .context(error::GetAtomNameReplySnafu)?
                                .name;
                            formats.push(String::from_utf8_lossy(&atom_name).clone().to_string());
                        }
                    }
                    return Ok(formats);
                }
                _ => continue,
            }
        }
        Ok(Vec::new())
    }

    pub fn display_name(&self) -> String {
        let display_name = self.display_name.as_deref().unwrap_or(":0");
        format!("display: {display_name}")
    }
}

impl AsRawFd for Context {
    fn as_raw_fd(&self) -> RawFd { self.connection.stream().as_raw_fd() }
}

#[derive(Debug)]
struct AtomCache {
    clipcat_clipboard: xproto::Atom,
    clipboard_manager: xproto::Atom,
    clipboard_selection: xproto::Atom,
    primary_selection: xproto::Atom,
    secondary_selection: xproto::Atom,
    targets: xproto::Atom,
}

impl AtomCache {
    fn new(conn: &impl Connection) -> Result<Self, Error> {
        Ok(Self {
            clipcat_clipboard: get_intern_atom(conn, b"CLIPCAT_CLIPBOARD")?,
            clipboard_manager: get_intern_atom(conn, b"CLIPBOARD_MANAGER")?,
            clipboard_selection: get_intern_atom(conn, b"CLIPBOARD")?,
            primary_selection: xproto::AtomEnum::PRIMARY.into(),
            secondary_selection: xproto::AtomEnum::SECONDARY.into(),
            targets: get_intern_atom(conn, b"TARGETS")?,
        })
    }
}

#[inline]
pub fn get_intern_atom(conn: &impl Connection, atom_name: &[u8]) -> Result<xproto::Atom, Error> {
    conn.intern_atom(false, atom_name)
        .with_context(|_| error::GetAtomIdentifierByNameSnafu {
            atom_name: String::from_utf8_lossy(atom_name).clone(),
        })?
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

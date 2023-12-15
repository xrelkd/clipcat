pub mod config;
mod entry;
mod filter;
mod kind;
pub mod serde;
pub mod utils;
mod watcher_state;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use bytes::Bytes;
use directories::ProjectDirs;
use lazy_static::lazy_static;

pub use self::{
    entry::{Entry as ClipEntry, Error as ClipEntryError, Metadata as ClipEntryMetadata},
    filter::Filter as ClipFilter,
    kind::Kind as ClipboardKind,
    watcher_state::WatcherState as ClipboardWatcherState,
};

pub const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    pub static ref PROJECT_SEMVER: semver::Version = semver::Version::parse(PROJECT_VERSION)
        .unwrap_or(semver::Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY
        });
}

pub const PROJECT_NAME: &str = "clipcat";
pub const PROJECT_NAME_WITH_INITIAL_CAPITAL: &str = "Clipcat";
pub const NOTIFICATION_SUMMARY: &str = "Clipcat - Clipboard Manager";

pub const DAEMON_PROGRAM_NAME: &str = "clipcatd";
pub const DAEMON_CONFIG_NAME: &str = "clipcatd.toml";
pub const DAEMON_HISTORY_FILE_NAME: &str = "clipcatd-history";

pub const CTL_PROGRAM_NAME: &str = "clipcatctl";
pub const CTL_CONFIG_NAME: &str = "clipcatctl.toml";

pub const MENU_PROGRAM_NAME: &str = "clipcat-menu";
pub const MENU_CONFIG_NAME: &str = "clipcat-menu.toml";

pub const NOTIFY_PROGRAM_NAME: &str = "clipcat-notify";

pub const DEFAULT_GRPC_PORT: u16 = 45045;
pub const DEFAULT_GRPC_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

pub const DEFAULT_WEBUI_PORT: u16 = 45046;
pub const DEFAULT_WEBUI_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

pub const DEFAULT_MENU_PROMPT: &str = "Clipcat";

lazy_static::lazy_static! {
pub static ref PROJECT_CONFIG_DIR: PathBuf = ProjectDirs::from("", PROJECT_NAME, PROJECT_NAME)
            .expect("Creating `ProjectDirs` should always success")
            .config_dir()
            .to_path_buf();
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ClipboardContent {
    Plaintext(String),
    Image { width: usize, height: usize, bytes: Bytes },
}

impl Default for ClipboardContent {
    fn default() -> Self { Self::Plaintext(String::new()) }
}

impl ClipboardContent {
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Plaintext(s) => s.is_empty(),
            Self::Image { bytes, .. } => bytes.is_empty(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Plaintext(s) => s.len(),
            Self::Image { bytes, .. } => bytes.len(),
        }
    }

    #[inline]
    pub const fn is_plaintext(&self) -> bool { matches!(&self, Self::Plaintext(_)) }

    #[inline]
    pub const fn is_image(&self) -> bool { matches!(&self, Self::Image { .. }) }

    #[inline]
    pub const fn mime(&self) -> mime::Mime {
        match self {
            Self::Plaintext(_) => mime::TEXT_PLAIN_UTF_8,
            Self::Image { .. } => mime::IMAGE_PNG,
        }
    }

    #[inline]
    pub fn basic_information(&self) -> String {
        let content_type = self.mime();
        let size = humansize::format_size(self.len(), humansize::BINARY);
        format!("{content_type}, {size}")
    }

    #[inline]
    #[must_use]
    pub fn id(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl AsRef<Self> for ClipboardContent {
    fn as_ref(&self) -> &Self { self }
}

use serde::{Deserialize, Serialize};
use zvariant::Type;

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Type,
)]
pub enum Kind {
    #[default]
    Clipboard,
    Primary,
    Secondary,
}

impl From<Kind> for clipcat_base::ClipboardKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Clipboard => Self::Clipboard,
            Kind::Primary => Self::Primary,
            Kind::Secondary => Self::Secondary,
        }
    }
}

impl From<clipcat_base::ClipboardKind> for Kind {
    fn from(kind: clipcat_base::ClipboardKind) -> Self {
        match kind {
            clipcat_base::ClipboardKind::Clipboard => Self::Clipboard,
            clipcat_base::ClipboardKind::Primary => Self::Primary,
            clipcat_base::ClipboardKind::Secondary => Self::Secondary,
        }
    }
}

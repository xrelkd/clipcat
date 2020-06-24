mod error;

#[cfg(feature = "external_editor")]
mod external;

pub use self::error::EditorError;

#[cfg(feature = "external_editor")]
pub use self::external::ExternalEditor;

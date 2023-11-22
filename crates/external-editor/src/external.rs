use std::{fmt, path::PathBuf, time::SystemTime};

use snafu::ResultExt;
use tokio::process::Command;

use crate::{error, error::Error};

#[allow(clippy::module_name_repetitions)]
pub struct ExternalEditor {
    editor: String,
}

impl ExternalEditor {
    pub fn new<S: fmt::Display>(editor: S) -> Self { Self { editor: editor.to_string() } }

    /// # Errors
    pub fn new_or_from_env<S: fmt::Display>(editor: Option<S>) -> Result<Self, Error> {
        if let Some(editor) = editor {
            return Ok(Self { editor: editor.to_string() });
        }

        Self::from_env()
    }

    /// # Errors
    pub fn from_env() -> Result<Self, Error> {
        let editor = std::env::var("EDITOR").context(error::GetEnvEditorSnafu)?;
        Ok(Self { editor })
    }

    /// # Errors
    /// # Panics
    ///
    /// This function panics while failed to get current timestamp.
    pub async fn execute(&self, data: &str) -> Result<String, Error> {
        let tmp_file = {
            let timestamp =
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("system time");

            [
                std::env::temp_dir(),
                PathBuf::from(format!(".{}-{:?}", clipcat::PROJECT_NAME, timestamp)),
            ]
            .into_iter()
            .collect::<PathBuf>()
        };

        tokio::fs::write(&tmp_file, data)
            .await
            .context(error::CreateTemporaryFileSnafu { filename: tmp_file.clone() })?;

        let _exit_status = Command::new(&self.editor)
            .arg(&tmp_file)
            .spawn()
            .context(error::CallExternalTextEditorSnafu { program: self.editor.clone() })?
            .wait()
            .await
            .context(error::ExecuteExternalTextEditorSnafu { program: self.editor.clone() })?;

        let data = tokio::fs::read_to_string(&tmp_file)
            .await
            .context(error::ReadTemporaryFileSnafu { filename: tmp_file.clone() })?;
        tokio::fs::remove_file(&tmp_file.clone())
            .await
            .context(error::RemoveTemporaryFileSnafu { filename: tmp_file.clone() })?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::ExternalEditor;

    #[test]
    fn test_from_env() {
        let external_editor = "my-editor";

        std::env::set_var("EDITOR", external_editor);
        let editor = ExternalEditor::from_env().unwrap();
        assert_eq!(&editor.editor, external_editor);

        std::env::remove_var("EDITOR");
        let editor = ExternalEditor::from_env();
        assert!(editor.is_err());
    }

    #[test]
    fn test_execute() {
        std::env::set_var("TMPDIR", "/tmp");

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let editor = ExternalEditor::new("echo");

        let data = "this is a string.\nЭто вох";
        let ret = runtime.block_on(async { editor.execute(data).await.unwrap() });
        assert_eq!(&ret, data);
    }
}

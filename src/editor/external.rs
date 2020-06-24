use std::time::SystemTime;

use snafu::ResultExt;
use tokio::process::Command;

use crate::editor::error::*;

pub struct ExternalEditor {
    editor: String,
}

impl ExternalEditor {
    pub fn new<S: ToString>(editor: S) -> ExternalEditor {
        ExternalEditor { editor: editor.to_string() }
    }

    pub fn new_or_from_env<S: ToString>(editor: Option<S>) -> Result<ExternalEditor, EditorError> {
        if let Some(editor) = editor {
            return Ok(ExternalEditor { editor: editor.to_string() });
        }

        Self::from_env()
    }

    pub fn from_env() -> Result<ExternalEditor, EditorError> {
        let editor = std::env::var("EDITOR").context(GetEnvEditor)?;
        Ok(ExternalEditor { editor })
    }

    pub async fn execute(&self, data: &str) -> Result<String, EditorError> {
        let tmp_file = {
            let timestamp =
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("system time");
            let filename = format!(".{}-{:?}", crate::PROJECT_NAME, timestamp);
            let mut path = std::env::temp_dir();
            path.push(filename);
            path
        };

        let _ = tokio::fs::write(&tmp_file, data)
            .await
            .context(CreateTemporaryFile { filename: tmp_file.to_owned() })?;

        Command::new(&self.editor)
            .arg(&tmp_file)
            .spawn()
            .context(CallExternalTextEditor { program: self.editor.to_owned() })?
            .await
            .context(ExecuteExternalTextEditor { program: self.editor.to_owned() })?;

        let data = tokio::fs::read_to_string(&tmp_file)
            .await
            .context(ReadTemporaryFile { filename: tmp_file.to_owned() })?;
        let _ = tokio::fs::remove_file(&tmp_file.to_owned())
            .await
            .context(RemoveTemporaryFile { filename: tmp_file.to_owned() })?;

        Ok(data)
    }
}

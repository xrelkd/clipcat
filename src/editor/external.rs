use std::time::SystemTime;

use snafu::ResultExt;
use tokio::process::Command;

use crate::editor::{error, error::EditorError};

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
        let editor = std::env::var("EDITOR").context(error::GetEnvEditor)?;
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

        tokio::fs::write(&tmp_file, data)
            .await
            .context(error::CreateTemporaryFile { filename: tmp_file.to_owned() })?;

        Command::new(&self.editor)
            .arg(&tmp_file)
            .spawn()
            .context(error::CallExternalTextEditor { program: self.editor.to_owned() })?
            .wait()
            .await
            .context(error::ExecuteExternalTextEditor { program: self.editor.to_owned() })?;

        let data = tokio::fs::read_to_string(&tmp_file)
            .await
            .context(error::ReadTemporaryFile { filename: tmp_file.to_owned() })?;
        tokio::fs::remove_file(&tmp_file.to_owned())
            .await
            .context(error::RemoveTemporaryFile { filename: tmp_file.to_owned() })?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::editor::ExternalEditor;

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

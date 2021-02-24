use std::io::Cursor;

use skim::prelude::*;
use snafu::ResultExt;

use clipcat::ClipEntry;

use crate::finder::{
    error, finder_stream::ENTRY_SEPARATOR, FinderError, FinderStream, SelectionMode,
};

pub struct BuiltinFinder;

impl BuiltinFinder {
    pub fn new() -> BuiltinFinder { BuiltinFinder }

    pub async fn select(
        &self,
        clips: &[ClipEntry],
        selection_mode: SelectionMode,
    ) -> Result<Vec<usize>, FinderError> {
        let input = self.generate_input(clips);

        let output = tokio::task::spawn_blocking(move || {
            let multi = match selection_mode {
                SelectionMode::Single => false,
                SelectionMode::Multiple => true,
            };
            let options =
                SkimOptionsBuilder::default().height(Some("100%")).multi(multi).build().unwrap();

            let item_reader = SkimItemReader::default();
            let items = item_reader.of_bufread(Cursor::new(input));

            // `run_with` would read and show items from the stream
            let selected_items = Skim::run_with(&options, Some(items))
                .map(|out| out.selected_items)
                .unwrap_or_else(Vec::new);

            selected_items.iter().map(|item| item.text()).collect::<Vec<_>>().join(ENTRY_SEPARATOR)
        })
        .await
        .context(error::JoinTask)?;

        Ok(self.parse_output(output.as_bytes()))
    }
}

impl FinderStream for BuiltinFinder {}

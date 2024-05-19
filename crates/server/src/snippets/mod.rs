mod event_handler;

use std::{collections::HashMap, path::PathBuf};

use clipcat_base::ClipEntry;
use notify::{RecursiveMode, Watcher};
use snafu::ResultExt;
use time::OffsetDateTime;

use self::event_handler::EventHandler;
pub use self::event_handler::{SnippetWatcherEvent, SnippetWatcherEventReceiver};
use crate::{config, error, error::Error};

async fn load(config: &config::SnippetConfig) -> HashMap<ClipEntry, Option<PathBuf>> {
    let (name, clip_contents) = match config {
        config::SnippetConfig::Inline { name, content } => {
            tracing::trace!("Load snippet `{name}`");
            (name, vec![(content.as_bytes().to_vec(), None)])
        }
        config::SnippetConfig::File { name, path } => {
            tracing::trace!("Load snippet `{name}` from file `{}`", path.display());
            let contents = tokio::fs::read(&path).await.map_or_else(
                |err| {
                    tracing::warn!(
                        "Failed to load snippet from `{}`, error: {err}",
                        path.display()
                    );
                    Vec::new()
                },
                |content| vec![(content, Some(path.clone()))],
            );
            (name, contents)
        }
        config::SnippetConfig::Directory { name, path } => {
            tracing::trace!("Load snippet `{name}` from directory `{}`", path.display());
            let contents = futures::future::join_all(
                clipcat_base::utils::fs::read_dir_recursively_async(&path)
                    .await
                    .into_iter()
                    .map(|file| (async move { (tokio::fs::read(&file).await.ok(), file) })),
            )
            .await
            .into_iter()
            .filter_map(|(c, _file_path)| c.map(|c| (c, Some(path.clone()))))
            .collect();
            (name, contents)
        }
    };

    if clip_contents.is_empty() {
        tracing::warn!("Snippet `{name}` is empty, ignored it");
        return HashMap::new();
    }

    clip_contents
        .into_iter()
        .filter_map(|(data, path)| {
            if data.is_empty() {
                tracing::warn!("Snippet `{name}` is empty, ignored it");
                return None;
            }

            if let Err(err) = simdutf8::basic::from_utf8(&data) {
                tracing::warn!("Snippet `{name}` is not valid UTF-8 string, error: {err}");
                return None;
            }

            ClipEntry::new(
                &data,
                &mime::TEXT_PLAIN_UTF_8,
                clipcat_base::ClipboardKind::Clipboard,
                Some(OffsetDateTime::UNIX_EPOCH),
            )
            .ok()
            .map(|clip| (clip, path))
        })
        .collect()
}

pub async fn load_and_create_watcher(
    snippets: &[config::SnippetConfig],
) -> Result<((notify::RecommendedWatcher, SnippetWatcherEventReceiver), Vec<ClipEntry>), Error> {
    let mut file_path_to_id = HashMap::new();
    let mut file_paths = Vec::new();
    let mut new_clips = Vec::new();
    for snippet in snippets {
        for (clip, file_path) in load(snippet).await {
            if let Some(file_path) = file_path {
                file_paths.push(file_path.clone());
                let _ = file_path_to_id.insert(file_path.clone(), clip.id());
            }
            new_clips.push(clip);
        }
    }

    let (event_handler, event_receiver) = EventHandler::new(file_path_to_id);
    let mut watcher =
        notify::recommended_watcher(event_handler).context(error::CreateFileWatcherSnafu)?;
    for file_path in file_paths {
        if let Err(err) = watcher.watch(&file_path, RecursiveMode::Recursive) {
            tracing::warn!("Could not watch file {}, error: {err}", file_path.display());
        }
    }
    Ok(((watcher, event_receiver), new_clips))
}

use std::{num::NonZeroUsize, time::Duration};

use clipcat_base::ClipEntryMetadata;
use clipcat_client::{Client, Manager};
use futures::StreamExt;
use lru::LruCache;
use snafu::ResultExt;
use tokio::io::AsyncWriteExt as _;

use crate::{
    cli::format_metadata_line,
    error::{self, Error},
};

const TAIL_BACKOFF_INITIAL: Duration = Duration::from_millis(100);
const TAIL_BACKOFF_MAX: Duration = Duration::from_secs(5);
const TAIL_PRINTED_IDS_CAP: NonZeroUsize = NonZeroUsize::new(1024).expect("1024 is non-zero");

pub async fn run(
    client: &Client,
    preview_length: usize,
    show_source_prefix: bool,
    no_id: bool,
    lines: u64,
    follow: bool,
) -> Result<i32, Error> {
    let mut state = TailState::new();
    if follow {
        loop {
            let maybe_result = run_session(
                client,
                no_id,
                lines,
                preview_length,
                show_source_prefix,
                &mut state,
                true,
            )
            .await;
            match maybe_result {
                Ok(()) => return Ok(0),
                Err(err) => {
                    let delay = state.next_backoff();
                    tracing::debug!(
                        "tail session ended ({err}); reconnecting in {} ms",
                        delay.as_millis()
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    } else {
        // Single-shot: fail fast if the daemon is unreachable.
        run_session(client, no_id, lines, preview_length, show_source_prefix, &mut state, false)
            .await
            .map(|()| 0)
    }
}

async fn run_session(
    client: &Client,
    no_id: bool,
    lines: u64,
    preview_length: usize,
    show_source_prefix: bool,
    state: &mut TailState,
    follow: bool,
) -> Result<(), Error> {
    // Subscribe before listing so events in the gap are buffered and deduped via
    // printed_ids. On reconnect, we skip listing entirely and only emit entries
    // from the new stream.
    let stream = if follow { Some(client.subscribe(preview_length).await?) } else { None };

    if state.is_first_session {
        let take = usize::try_from(lines).unwrap_or(0);
        if take > 0 {
            let snapshot = client.list(preview_length).await?;

            // `client.list` returns newest first. We take the most recent `take` entries
            // and iterate in reverse so output reads oldest -> newest, matching the
            // chronological order of streamed events under `-f`.
            let recent: Vec<_> = snapshot.into_iter().take(take).collect();
            for metadata in recent.iter().rev() {
                if state.printed_ids.contains(&metadata.id) {
                    continue;
                }
                write_metadata_line(metadata, no_id, show_source_prefix).await?;
                let _ = state.printed_ids.put(metadata.id, ());
            }
        }
        state.is_first_session = false;
    }

    let Some(mut stream) = stream else {
        return Ok(());
    };
    while let Some(item) = stream.next().await {
        let metadata = item?;
        if state.printed_ids.contains(&metadata.id) {
            continue;
        }
        write_metadata_line(&metadata, no_id, show_source_prefix).await?;
        let _ = state.printed_ids.put(metadata.id, ());
        state.reset_backoff();
    }

    Err(Error::Operation { error: "subscription stream ended".to_owned() })
}

async fn write_metadata_line(
    metadata: &ClipEntryMetadata,
    no_id: bool,
    show_source_prefix: bool,
) -> Result<(), Error> {
    let line = format_metadata_line(metadata, no_id, show_source_prefix);
    tokio::io::stdout().write_all(line.as_bytes()).await.context(error::WriteStdoutSnafu)?;
    Ok(())
}

struct TailState {
    printed_ids: LruCache<u64, ()>,
    is_first_session: bool,
    backoff: Duration,
}

impl TailState {
    fn new() -> Self {
        Self {
            printed_ids: LruCache::new(TAIL_PRINTED_IDS_CAP),
            is_first_session: true,
            backoff: TAIL_BACKOFF_INITIAL,
        }
    }

    fn next_backoff(&mut self) -> Duration {
        let current = self.backoff;
        self.backoff = std::cmp::min(self.backoff.saturating_mul(2), TAIL_BACKOFF_MAX);
        current
    }

    const fn reset_backoff(&mut self) { self.backoff = TAIL_BACKOFF_INITIAL; }
}

mod error;

use std::{io::Write, sync::Arc};

use clap::{CommandFactory, Parser, Subcommand};
use clipcat_base::{ClipFilter, ClipboardKind};
use serde::Serialize;
use snafu::ResultExt;
use time::OffsetDateTime;
use tokio::runtime::Runtime;

use self::error::Error;

#[derive(Parser)]
#[clap(name = clipcat_base::NOTIFY_PROGRAM_NAME, author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    commands: Option<Commands>,

    #[clap(long = "no-clipboard", help = "Do not listen clipboard")]
    no_clipboard: bool,

    #[clap(long = "no-primary", help = "Do not listen primary")]
    no_primary: bool,

    #[clap(long = "no-secondary", help = "Do not listen secondary")]
    no_secondary: bool,
}

#[derive(Clone, Subcommand)]
enum Commands {
    #[clap(about = "Print version information")]
    Version,

    #[clap(about = "Output shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },
}

impl Cli {
    fn run(self) -> Result<(), Error> {
        match self.commands {
            Some(Commands::Version) => {
                std::io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("failed to write to stdout");
                Ok(())
            }
            Some(Commands::Completions { shell }) => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                Ok(())
            }
            None => {
                let enable_clipboard = !self.no_clipboard;
                let enable_primary = !self.no_primary;
                let enable_secondary = !self.no_secondary;

                if !enable_clipboard && !enable_primary && !enable_secondary {
                    return Err(Error::ListenToNothing);
                }

                Runtime::new().context(error::InitializeTokioRuntimeSnafu)?.block_on(
                    async move {
                        let backend =
                            clipcat_server::backend::new(&Arc::new(ClipFilter::default()), &[])
                                .context(error::InitializeClipboardBackendSnafu)?;
                        let mut subscriber =
                            backend.subscribe().context(error::SubscribeClipboardSnafu)?;

                        while let Some((kind, mime)) = subscriber.next().await {
                            match kind {
                                ClipboardKind::Clipboard if enable_clipboard => {}
                                ClipboardKind::Primary if enable_primary => {}
                                ClipboardKind::Secondary if enable_secondary => {}
                                _ => continue,
                            }

                            let info = {
                                let now = OffsetDateTime::now_local()
                                    .unwrap_or_else(|_| OffsetDateTime::now_utc());
                                ClipInfo { kind, mime, timestamp: now }
                            };
                            serde_json::to_writer_pretty(std::io::stdout(), &info)
                                .expect("`ClipInfo` is serializable");
                            return Ok(());
                        }

                        drop(backend);

                        Err(Error::WaitForClipboardEvent)
                    },
                )?;

                Ok(())
            }
        }
    }
}

#[derive(Serialize)]
struct ClipInfo {
    #[serde(rename = "clipboard_kind", with = "clipcat_base::serde::clipboard_kind")]
    kind: ClipboardKind,

    #[serde(with = "clipcat_base::serde::mime")]
    mime: mime::Mime,

    #[serde(with = "time::serde::rfc3339")]
    timestamp: OffsetDateTime,
}

fn main() {
    if let Err(err) = Cli::parse().run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

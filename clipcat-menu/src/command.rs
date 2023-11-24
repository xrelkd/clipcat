use std::{io::Write, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use clipcat::{ClipEntry, ClipboardKind};
use clipcat_client::{Client, Manager};
use clipcat_external_editor::ExternalEditor;
use snafu::ResultExt;
use tokio::runtime::Runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    error::{self, Error},
    finder::{FinderRunner, FinderType},
};

const LINE_LENGTH: usize = 100;

#[derive(Debug, Parser)]
#[clap(name = clipcat::MENU_PROGRAM_NAME, author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Option<Commands>,

    #[clap(long = "config", short = 'c', help = "Specify a configuration file")]
    config_file: Option<PathBuf>,

    #[clap(long, short = 'f', help = "Specify a finder")]
    finder: Option<FinderType>,

    #[clap(long, short = 'm', help = "Specify the menu length of finder")]
    menu_length: Option<usize>,

    #[clap(long, short = 'l', help = "Specify the length of a line showing on finder")]
    line_length: Option<usize>,

    #[clap(long = "log-level", help = "Specify a log level")]
    log_level: Option<tracing::Level>,
}

#[allow(variant_size_differences)]
#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Print version information")]
    Version,

    #[clap(about = "Output shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },

    #[clap(about = "Output default configuration")]
    DefaultConfig,

    #[clap(about = "Print available text finders")]
    ListFinder,

    #[clap(about = "Insert selected clip into clipboard")]
    Insert {
        #[clap(
            long = "kind",
            short = 'k',
            default_value = "clipboard",
            help = "Specify which clipboard to insert (\"clipboard\", \"primary\", \"secondary\")"
        )]
        kind: ClipboardKind,
    },

    #[clap(
        aliases = &["rm", "delete", "del"],
        about = "Remove selected clip")]
    Remove,

    #[clap(about = "Edit selected clip")]
    Edit {
        #[clap(env = "EDITOR", long = "editor", short = 'e', help = "Specify a external editor")]
        editor: String,
    },
}

impl Cli {
    pub fn new() -> Self { Self::parse() }

    pub fn run(self) -> Result<(), Error> {
        let Self { commands, config_file, finder, menu_length, line_length, log_level } = self;

        match commands {
            Some(Commands::Version) => {
                std::io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("failed to write to stdout");

                return Ok(());
            }
            Some(Commands::Completions { shell }) => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());

                return Ok(());
            }
            Some(Commands::DefaultConfig) => {
                let config_text =
                    toml::to_string_pretty(&Config::default()).expect("Config is serializable");
                std::io::stdout()
                    .write_all(config_text.as_bytes())
                    .expect("Failed to write to stdout");
                return Ok(());
            }
            Some(Commands::ListFinder) => {
                for ty in FinderType::available_types() {
                    println!("{ty}");
                }
                return Ok(());
            }
            _ => {}
        }

        let mut config = Config::load_or_default(config_file.unwrap_or_else(Config::default_path));
        if let Some(log_level) = log_level {
            config.log_level = log_level;
        }

        init_tracing(config.log_level);

        let finder = {
            if let Some(finder) = finder {
                config.finder = finder;
            }

            let mut finder = FinderRunner::from_config(&config);
            if let Some(line_length) = line_length {
                finder.set_line_length(line_length);
            }

            if let Some(menu_length) = menu_length {
                finder.set_menu_length(menu_length);
            }
            finder
        };

        let fut = async move {
            let client = {
                let grpc_endpoint: http::Uri =
                    format!("http://{server_addr}", server_addr = config.server_socket_address())
                        .parse()
                        .expect("valid uri");
                Client::new(clipcat_client::Config { grpc_endpoint }).await?
            };

            let clips = client.list().await?;

            match commands {
                Some(Commands::Insert { kind }) => {
                    insert_clip(&clips, finder, &client, kind).await?;
                }
                None => insert_clip(&clips, finder, &client, ClipboardKind::Clipboard).await?,
                Some(Commands::Remove) => {
                    let selections = finder.multiple_select(&clips).await?;
                    let ids: Vec<_> = selections.into_iter().map(|(_, clip)| clip.id()).collect();
                    let removed_ids = client.batch_remove(&ids).await?;
                    for id in removed_ids {
                        tracing::info!("Removing clip (id: {:016x})", id);
                    }
                }
                Some(Commands::Edit { editor }) => {
                    let selection = finder.single_select(&clips).await?;
                    if let Some((_index, clip)) = selection {
                        if clip.is_utf8_string() {
                            let editor = ExternalEditor::new(editor);
                            let new_data = editor
                                .execute(&clip.as_utf8_string())
                                .await
                                .context(error::CallEditorSnafu)?;
                            let (ok, new_id) =
                                client.update(clip.id(), new_data.as_bytes(), clip.mime()).await?;
                            if ok {
                                tracing::info!("Editing clip (id: {:016x})", new_id);
                            }
                            let _ok = client.mark(new_id, ClipboardKind::Clipboard).await?;
                            drop(client);
                        }
                    } else {
                        tracing::info!("Nothing is selected");
                        return Ok(());
                    }
                }
                _ => unreachable!(),
            }

            Ok(())
        };

        Runtime::new().context(error::InitializeTokioRuntimeSnafu)?.block_on(fut)
    }
}

async fn insert_clip(
    clips: &[ClipEntry],
    finder: FinderRunner,
    client: &Client,
    clipboard_kind: ClipboardKind,
) -> Result<(), Error> {
    let selection = finder.single_select(clips).await?;
    if let Some((index, clip)) = selection {
        tracing::info!(
            "Inserting clip (index: {}, id: {:016x}, content: {:?})",
            index,
            clip.id(),
            clip.printable_data(Some(LINE_LENGTH)),
        );
        let _ok = client.mark(clip.id(), clipboard_kind).await?;
    } else {
        tracing::info!("Nothing is selected");
    }

    Ok(())
}

fn init_tracing(log_level: tracing::Level) {
    // filter
    let filter_layer = tracing_subscriber::filter::LevelFilter::from_level(log_level);

    // format
    let fmt_layer =
        tracing_subscriber::fmt::layer().pretty().with_thread_ids(true).with_thread_names(true);

    // subscriber
    let registry = tracing_subscriber::registry().with(filter_layer).with(fmt_layer);
    match tracing_journald::layer() {
        Ok(layer) => registry.with(layer).init(),
        Err(_err) => {
            registry.init();
        }
    }
}

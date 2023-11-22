use std::{io::Write, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use clipcat::{ClipEntry, ClipboardMode};
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
#[clap(name = clipcat::MENU_PROGRAM_NAME)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Option<Commands>,

    #[clap(long = "config", short = 'c', help = "Specifies a configuration file")]
    config_file: Option<PathBuf>,

    #[clap(long, short = 'f', help = "Specifies a finder")]
    finder: Option<FinderType>,

    #[clap(long, short = 'm', help = "Specifies the menu length of finder")]
    menu_length: Option<usize>,

    #[clap(long, short = 'l', help = "Specifies the length of a line showing on finder")]
    line_length: Option<usize>,
}

#[allow(variant_size_differences)]
#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Prints version information")]
    Version,

    #[clap(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },

    #[clap(about = "Outputs default configuration")]
    DefaultConfig,

    #[clap(about = "Prints available text finders")]
    ListFinder,

    #[clap(about = "Insert selected clip into clipboard")]
    Insert {
        #[clap(
            long = "mode",
            short = 'm',
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"selection\")"
        )]
        mode: ClipboardMode,
    },

    #[clap(
        aliases = &["rm", "delete", "del"],
        about = "Removes selected clip")]
    Remove,

    #[clap(about = "Edit selected clip")]
    Edit {
        #[clap(env = "EDITOR", long = "editor", short = 'e', help = "Specifies a external editor")]
        editor: String,
    },
}

impl Cli {
    pub fn new() -> Self { Self::parse() }

    pub fn run(self) -> Result<(), Error> {
        match self.subcommand {
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

        init_tracing();

        let mut config =
            Config::load_or_default(self.config_file.unwrap_or_else(Config::default_path));

        let finder = {
            if let Some(finder) = self.finder {
                config.finder = finder;
            }

            let mut finder = FinderRunner::from_config(&config);
            if let Some(line_length) = self.line_length {
                finder.set_line_length(line_length);
            }

            if let Some(menu_length) = self.menu_length {
                finder.set_menu_length(menu_length);
            }
            finder
        };

        let subcommand = self.subcommand;
        let fut = async move {
            let Config { server_host, server_port, .. } = config;

            let client = {
                let grpc_endpoint: http::Uri =
                    format!("http://{server_host}:{server_port}").parse().expect("valid uri");
                Client::new(clipcat_client::Config { grpc_endpoint }).await?
            };

            let clips = client.list().await?;

            match subcommand {
                Some(Commands::Insert { mode }) => {
                    insert_clip(&clips, finder, &client, mode).await?;
                }
                None => insert_clip(&clips, finder, &client, ClipboardMode::Clipboard).await?,
                Some(Commands::Remove) => {
                    let selections = finder.multiple_select(&clips).await?;
                    let ids: Vec<_> = selections.into_iter().map(|(_, clip)| clip.id).collect();
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
                                client.update(clip.id, new_data.as_bytes(), clip.mime).await?;
                            if ok {
                                tracing::info!("Editing clip (id: {:016x})", new_id);
                            }
                            let _ok = client.mark(new_id, ClipboardMode::Clipboard).await?;
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
    clipboard_mode: ClipboardMode,
) -> Result<(), Error> {
    let selection = finder.single_select(clips).await?;
    if let Some((index, clip)) = selection {
        tracing::info!(
            "Inserting clip (index: {}, id: {:016x}, content: {:?})",
            index,
            clip.id,
            clip.printable_data(Some(LINE_LENGTH)),
        );
        let _ok = client.mark(clip.id, clipboard_mode).await?;
    } else {
        tracing::info!("Nothing is selected");
    }

    Ok(())
}

fn init_tracing() {
    // filter
    let filter_layer = tracing_subscriber::filter::LevelFilter::from_level(tracing::Level::INFO);

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

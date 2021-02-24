use std::path::PathBuf;

use snafu::ResultExt;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use clipcat::{editor::ExternalEditor, grpc::GrpcClient, ClipEntry, ClipboardMode};

use crate::{
    config::Config,
    error::{self, Error},
    finder::{FinderRunner, FinderType},
};

const LINE_LENGTH: usize = 100;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = clipcat::MENU_PROGRAM_NAME)]
pub struct Command {
    #[structopt(subcommand)]
    subcommand: Option<SubCommand>,

    #[structopt(long = "config", short = "c", help = "Specifies a configuration file")]
    config_file: Option<PathBuf>,

    #[structopt(long, short = "f", help = "Specifies a finder")]
    finder: Option<FinderType>,

    #[structopt(long, short = "m", help = "Specifies the menu length of finder")]
    menu_length: Option<usize>,

    #[structopt(long, short = "l", help = "Specifies the length of a line showing on finder")]
    line_length: Option<usize>,
}

#[derive(Debug, Clone, StructOpt)]
pub enum SubCommand {
    #[structopt(about = "Prints version information")]
    Version,

    #[structopt(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: structopt::clap::Shell },

    #[structopt(about = "Outputs default configuration")]
    DefaultConfig,

    #[structopt(about = "Prints available text finders")]
    ListFinder,

    #[structopt(about = "Insert selected clip into clipboard")]
    Insert {
        #[structopt(
            short = "m",
            long = "mode",
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"selection\")"
        )]
        mode: ClipboardMode,
    },

    #[structopt(
        aliases = &["rm", "delete", "del"],
        about = "Removes selected clip")]
    Remove,

    #[structopt(about = "Edit selected clip")]
    Edit {
        #[structopt(
            env = "EDITOR",
            long = "editor",
            short = "e",
            help = "Specifies a external editor"
        )]
        editor: String,
    },
}

impl Command {
    pub fn new() -> Command { Command::from_args() }

    pub fn run(self) -> Result<(), Error> {
        match self.subcommand {
            Some(SubCommand::Version) => {
                Self::clap()
                    .write_long_version(&mut std::io::stdout())
                    .expect("failed to write to stdout");
                return Ok(());
            }
            Some(SubCommand::Completions { shell }) => {
                Self::clap().gen_completions_to(
                    clipcat::MENU_PROGRAM_NAME,
                    shell,
                    &mut std::io::stdout(),
                );
                return Ok(());
            }
            Some(SubCommand::DefaultConfig) => {
                println!(
                    "{}",
                    toml::to_string_pretty(&Config::default()).expect("Config is serializable")
                );
                return Ok(());
            }
            Some(SubCommand::ListFinder) => {
                for ty in FinderType::available_types() {
                    println!("{}", ty.to_string());
                }
                return Ok(());
            }
            _ => {}
        }

        {
            use tracing_subscriber::prelude::*;

            let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
            let level_filter = tracing_subscriber::filter::LevelFilter::INFO;

            let registry = tracing_subscriber::registry().with(level_filter).with(fmt_layer);
            match tracing_journald::layer() {
                Ok(layer) => registry.with(layer).init(),
                Err(_err) => {
                    registry.init();
                }
            }
        }

        let mut config =
            Config::load_or_default(&self.config_file.unwrap_or_else(Config::default_path));

        let finder = {
            if let Some(finder) = self.finder {
                config.finder = finder;
            }

            let mut finder = FinderRunner::from_config(&config)?;
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
            let grpc_addr = format!("http://{}:{}", config.server_host, config.server_port);
            let mut client = GrpcClient::new(grpc_addr).await?;
            let clips = client.list().await?;

            match subcommand {
                Some(SubCommand::Insert { mode }) => {
                    insert_clip(&clips, finder, client, mode).await?
                }
                None => insert_clip(&clips, finder, client, ClipboardMode::Clipboard).await?,
                Some(SubCommand::Remove) => {
                    let selections = finder.multiple_select(&clips).await?;
                    let ids: Vec<_> = selections.into_iter().map(|(_, clip)| clip.id).collect();
                    let removed_ids = client.batch_remove(&ids).await?;
                    for id in removed_ids {
                        tracing::info!("Removing clip (id: {:016x})", id);
                    }
                }
                Some(SubCommand::Edit { editor }) => {
                    let selection = finder.single_select(&clips).await?;
                    if let Some((_index, clip)) = selection {
                        if clip.is_utf8_string() {
                            let editor = ExternalEditor::new(editor);
                            let new_data = editor
                                .execute(&clip.as_utf8_string())
                                .await
                                .context(error::CallEditor)?;
                            let (ok, new_id) =
                                client.update(clip.id, new_data.as_bytes(), clip.mime).await?;
                            if ok {
                                tracing::info!("Editing clip (id: {:016x})", new_id);
                            }
                            client.mark(new_id, ClipboardMode::Clipboard).await?;
                        } else {
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

        let runtime = Runtime::new().context(error::CreateTokioRuntime)?;
        runtime.block_on(fut)
    }
}

async fn insert_clip(
    clips: &[ClipEntry],
    finder: FinderRunner,
    mut client: GrpcClient,
    clipboard_mode: ClipboardMode,
) -> Result<(), Error> {
    let selection = finder.single_select(&clips).await?;
    if let Some((index, clip)) = selection {
        tracing::info!(
            "Inserting clip (index: {}, id: {:016x}, content: {:?})",
            index,
            clip.id,
            clip.printable_data(Some(LINE_LENGTH)),
        );
        client.mark(clip.id, clipboard_mode).await?;
    } else {
        tracing::info!("Nothing is selected");
    }

    Ok(())
}

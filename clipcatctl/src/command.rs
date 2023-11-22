use std::{io::Write, num::ParseIntError, path::PathBuf, str::FromStr};

use clap::{CommandFactory, Parser, Subcommand};
use clipcat::{ClipboardKind, ClipboardWatcherState};
use clipcat_client::{Client, Manager as _, Watcher as _};
use clipcat_external_editor::ExternalEditor;
use snafu::ResultExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    runtime::Runtime,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    error::{self, Error},
};

#[derive(Parser)]
#[clap(name = clipcat::CTL_PROGRAM_NAME)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Option<Commands>,

    #[clap(long = "config", short = 'c', help = "Specifies a configuration file")]
    config_file: Option<PathBuf>,

    #[clap(long = "host", short = 'h', help = "Specifies a server host")]
    server_host: Option<std::net::IpAddr>,

    #[clap(long = "port", short = 'p', help = "Specifies a server port")]
    server_port: Option<u16>,

    #[clap(long = "log-level", help = "Specifies a log level")]
    log_level: Option<tracing::Level>,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    #[clap(about = "Prints version information")]
    Version,

    #[clap(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },

    #[clap(about = "Outputs default configuration")]
    DefaultConfig,

    #[clap(aliases = &["paste"], about = "Insert new clip into clipboard")]
    Insert {
        #[clap(
            long = "kind",
            short = 'k',
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"primary\", \
                    \"secondary\")"
        )]
        kind: ClipboardKind,

        data: String,
    },

    #[clap(aliases = &["cut"], about = "Loads file into clipboard")]
    Load {
        #[clap(
            long = "kind",
            short = 'k',
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"primary\", \
                    \"secondary\")"
        )]
        kind: ClipboardKind,

        #[clap(long = "file", short = 'f')]
        file_path: Option<PathBuf>,
    },

    #[clap(aliases = &["paste"], about = "Pastes content of current clipboard into file")]
    Save {
        #[clap(
            long = "kind",
            short = 'k',
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"primary\", \
                    \"secondary\")"
        )]
        kind: ClipboardKind,

        #[clap(long = "file", short = 'f')]
        file_path: Option<PathBuf>,
    },

    #[clap(about = "Prints clip with <id>")]
    Get {
        #[clap(value_parser= parse_hex )]
        id: Option<u64>,
    },

    #[clap(
        aliases = &["ls"],
        about = "Prints history of clipboard")]
    List {
        #[clap(long)]
        no_id: bool,
    },

    #[clap(about = "Updates clip with <id>")]
    Update {
        #[clap(value_parser= parse_hex )]
        id: u64,
        data: String,
    },

    #[clap(about = "Edits clip with <id>")]
    Edit {
        #[clap(env = "EDITOR", long = "editor", short = 'e')]
        editor: String,

        #[clap(value_parser= parse_hex )]
        id: u64,
    },

    #[clap(
        aliases = &["rm", "delete", "del"],
        about = "Removes clips with [ids]")]
    Remove { ids: Vec<String> },

    #[clap(name = "promote", about = "Replaces content of clipboard with clip with <id>")]
    Mark {
        #[clap(
            long = "kind",
            short = 'k',
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"primary\", \
                    \"secondary\")"
        )]
        kind: ClipboardKind,

        #[clap(value_parser = parse_hex )]
        id: u64,
    },

    #[clap(
        aliases = &["remove-all"],
        about = "Removes all clips in clipboard"
    )]
    Clear,

    #[clap(
        aliases = &["count", "len"],
        about = "Prints length of clipboard history")]
    Length,

    #[clap(aliases = &["enable"], about = "Enable clipboard watcher")]
    EnableWatcher,

    #[clap(aliases = &["disable"], about = "Disable clipboard watcher")]
    DisableWatcher,

    #[clap(aliases = &["toggle"], about = "Toggle clipboard watcher")]
    ToggleWatcher,

    #[clap(aliases = &["watcher-state"], about = "Get clipboard watcher state")]
    GetWatcherState,
}

impl Cli {
    pub fn new() -> Self { Self::parse() }

    fn load_config(&self) -> Config {
        let mut config =
            Config::load_or_default(self.config_file.clone().unwrap_or_else(Config::default_path));
        if let Some(host) = self.server_host {
            config.server_host = host;
        }

        if let Some(port) = self.server_port {
            config.server_port = port;
        }

        if let Ok(log_level) = std::env::var("RUST_LOG") {
            config.log_level = tracing::Level::from_str(&log_level).unwrap_or(tracing::Level::INFO);
        }

        if let Some(log_level) = self.log_level {
            config.log_level = log_level;
        }

        config
    }

    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> Result<i32, Error> {
        match self.subcommand {
            Some(Commands::Version) => {
                std::io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("failed to write to stdout");

                return Ok(0);
            }
            Some(Commands::Completions { shell }) => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                return Ok(0);
            }
            Some(Commands::DefaultConfig) => {
                let config_text =
                    toml::to_string_pretty(&Config::default()).expect("Config is serializable");
                std::io::stdout()
                    .write_all(config_text.as_bytes())
                    .expect("Failed to write to stdout");
                return Ok(0);
            }
            _ => {}
        }

        let Config { server_host, server_port, log_level } = self.load_config();
        init_tracing(log_level);

        let fut = async move {
            let client = {
                let grpc_endpoint: http::Uri =
                    format!("http://{server_host}:{server_port}").parse().expect("valid uri");
                Client::new(clipcat_client::Config { grpc_endpoint }).await?
            };

            match self.subcommand {
                None => {
                    print_list(&client, false).await?;
                }
                Some(Commands::List { no_id }) => {
                    print_list(&client, no_id).await?;
                }
                Some(Commands::Get { id }) => {
                    let data = if let Some(id) = id {
                        client.get(id).await?
                    } else {
                        client
                            .list()
                            .await?
                            .into_iter()
                            .find(|entry| entry.kind() == ClipboardKind::Clipboard)
                            .unwrap_or_default()
                    };

                    println!("{}", data.printable_data(None));
                }
                Some(Commands::Insert { kind, data }) => {
                    let _ = client.insert(data.as_bytes(), mime::TEXT_PLAIN_UTF_8, kind).await?;
                }
                Some(Commands::Length) => {
                    println!("{len}", len = client.length().await?);
                }
                Some(Commands::Load { file_path, kind }) => {
                    let data = load_file_or_read_stdin(file_path).await?;
                    let _ = client.insert(data.as_bytes(), mime::TEXT_PLAIN_UTF_8, kind).await?;
                }
                Some(Commands::Save { file_path, kind }) => {
                    let data = client.get_current_clip(kind).await?;
                    save_file_or_write_stdout(file_path, data.as_bytes()).await?;
                }
                Some(Commands::Remove { ids }) => {
                    let ids = ids
                        .into_iter()
                        .filter_map(|id| match parse_hex(&id) {
                            Ok(id) => Some(id),
                            Err(err) => {
                                eprintln!("Failed to parse ID {id}, error: {err}");
                                None
                            }
                        })
                        .collect::<Vec<u64>>();

                    if ids.is_empty() {
                        println!("Nothing is removed");
                        return Ok(0);
                    }
                    drop(client.batch_remove(&ids).await?);
                }
                Some(Commands::Clear) => {
                    client.clear().await?;
                }
                Some(Commands::Edit { id, editor }) => {
                    let data = client.get(id).await?;
                    if data.is_utf8_string() {
                        let editor = ExternalEditor::new(editor);
                        let data = editor
                            .execute(&data.as_utf8_string())
                            .await
                            .context(error::CallEditorSnafu)?;
                        let (ok, new_id) =
                            client.update(id, data.as_bytes(), mime::TEXT_PLAIN_UTF_8).await?;
                        if ok {
                            println!("{new_id:016x}");
                        }
                        let _ok = client.mark(new_id, ClipboardKind::Clipboard).await?;
                    } else {
                        println!(
                            "{:016x} is a {}, you could not edit with text editor",
                            id,
                            data.mime().essence_str()
                        );
                    }
                }
                Some(Commands::Update { id, data }) => {
                    let (ok, new_id) =
                        client.update(id, data.as_bytes(), mime::TEXT_PLAIN_UTF_8).await?;
                    if ok {
                        println!("{new_id:016x}");
                    }
                }
                Some(Commands::Mark { id, kind }) => {
                    if client.mark(id, kind).await? {
                        println!("Ok");
                    }
                }
                Some(Commands::EnableWatcher) => {
                    print_watcher_state(client.enable_watcher().await?);
                }
                Some(Commands::DisableWatcher) => {
                    print_watcher_state(client.disable_watcher().await?);
                }
                Some(Commands::ToggleWatcher) => {
                    print_watcher_state(client.toggle_watcher().await?);
                }
                Some(Commands::GetWatcherState) => {
                    print_watcher_state(client.get_watcher_state().await?);
                }
                _ => unreachable!(),
            }

            drop(client);
            Ok(0)
        };

        Runtime::new().context(error::InitializeTokioRuntimeSnafu)?.block_on(fut)
    }
}

#[inline]
fn parse_hex(src: &str) -> Result<u64, ParseIntError> { u64::from_str_radix(src, 16) }

async fn print_list(client: &Client, no_id: bool) -> Result<(), Error> {
    const LINE_LENGTH: Option<usize> = Some(100);

    let list = client.list().await?;
    for data in list {
        let content = data.printable_data(LINE_LENGTH);
        if no_id {
            println!("{content}");
        } else {
            println!("{:016x}: {content}", data.id());
        }
    }
    Ok(())
}

async fn load_file_or_read_stdin(file_path: Option<PathBuf>) -> Result<String, Error> {
    if let Some(file_path) = file_path {
        tokio::fs::read_to_string(&file_path)
            .await
            .context(error::ReadFileSnafu { filename: file_path.clone() })
    } else {
        let mut data = String::new();
        let _size =
            tokio::io::stdin().read_to_string(&mut data).await.context(error::ReadStdinSnafu)?;
        Ok(data)
    }
}

async fn save_file_or_write_stdout<Data>(
    file_path: Option<PathBuf>,
    data: Data,
) -> Result<(), Error>
where
    Data: AsRef<[u8]> + Send + Unpin,
{
    if let Some(file_path) = file_path {
        tokio::fs::write(&file_path, data)
            .await
            .context(error::ReadFileSnafu { filename: file_path.clone() })
    } else {
        tokio::io::stdout().write_all(data.as_ref()).await.context(error::WriteStdoutSnafu)
    }
}

#[inline]
fn print_watcher_state(state: ClipboardWatcherState) {
    let msg = match state {
        ClipboardWatcherState::Enabled => "Clipboard watcher is running",
        ClipboardWatcherState::Disabled => "Clipboard watcher is not running",
    };
    println!("{msg}");
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

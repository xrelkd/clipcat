use std::{io::Write, num::ParseIntError, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use clipcat_base::{ClipEntryMetadata, ClipboardKind, ClipboardWatcherState};
use clipcat_client::{Client, Manager as _, Watcher as _};
use clipcat_external_editor::ExternalEditor;
use snafu::ResultExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    runtime::Runtime,
};

use crate::{
    config::Config,
    error::{self, Error},
};

const PREVIEW_LENGTH: usize = 100;

#[derive(Parser)]
#[clap(name = clipcat_base::CTL_PROGRAM_NAME, author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Option<Commands>,

    #[clap(
        long = "config",
        short = 'c',
        env = "CLIPCATCTL_CONFIG_FILE_PATH",
        help = "Specify a configuration file"
    )]
    config_file: Option<PathBuf>,

    #[clap(
        long = "server-endpoint",
        env = "CLIPCATCTL_SERVER_ENDPOINT",
        help = "Specify a server endpoint"
    )]
    server_endpoint: Option<http::Uri>,

    #[clap(long = "log-level", env = "CLIPCATCTL_LOG_LEVEL", help = "Specify a log level")]
    log_level: Option<tracing::Level>,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    #[clap(about = "Print version information")]
    Version,

    #[clap(about = "Output shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },

    #[clap(about = "Output default configuration")]
    DefaultConfig,

    #[clap(about = "Insert new clip into clipboard")]
    Insert {
        #[clap(
            long = "kind",
            short = 'k',
            default_value = "clipboard",
            help = "Specify which clipboard to insert (\"clipboard\", \"primary\", \"secondary\")"
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
            help = "Specify which clipboard to insert (\"clipboard\", \"primary\", \"secondary\")"
        )]
        kind: ClipboardKind,

        #[clap(
            long = "mime",
            short = 'm',
            default_value = "text/plain; charset=utf-8",
            help = "Specify the MIME type of the content"
        )]
        mime: mime::Mime,

        #[clap(long = "file", short = 'f')]
        file_path: Option<PathBuf>,
    },

    #[clap(aliases = &["paste"], about = "Paste content of current clipboard into file")]
    Save {
        #[clap(
            long = "kind",
            short = 'k',
            default_value = "clipboard",
            help = "Specify which clipboard to extract (\"clipboard\", \"primary\", \"secondary\")"
        )]
        kind: ClipboardKind,

        #[clap(long = "file", short = 'f')]
        file_path: Option<PathBuf>,
    },

    #[clap(about = "Print clip with <id>")]
    Get {
        #[clap(value_parser= parse_hex )]
        id: Option<u64>,
    },

    #[clap(
        aliases = &["ls"],
        about = "Print history of clipboard"
    )]
    List {
        #[clap(long)]
        no_id: bool,
    },

    #[clap(about = "Update clip with <id>")]
    Update {
        #[clap(value_parser= parse_hex )]
        id: u64,
        data: String,
    },

    #[clap(about = "Edit clip with <id>")]
    Edit {
        #[clap(env = "EDITOR", long = "editor", short = 'e')]
        editor: String,

        #[clap(value_parser= parse_hex )]
        id: u64,
    },

    #[clap(
        aliases = &["rm", "delete", "del"],
        about = "Remove clips with [ids]"
    )]
    Remove { ids: Vec<String> },

    #[clap(name = "promote", about = "Replace content of clipboard with clip with <id>")]
    Mark {
        #[clap(
            long = "kinds",
            short = 'k',
            default_value = "clipboard",
            help = "Specify which clipboard to insert (\"clipboard\", \"primary\", \"secondary\")"
        )]
        kinds: Vec<ClipboardKind>,

        #[clap(value_parser = parse_hex )]
        id: u64,
    },

    #[clap(
        aliases = &["remove-all"],
        about = "Remove all clips in clipboard"
    )]
    Clear,

    #[clap(
        aliases = &["count", "len"],
        about = "Print length of clipboard history"
    )]
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
        if let Some(endpoint) = &self.server_endpoint {
            config.server_endpoint = endpoint.clone();
        }

        if let Some(log_level) = self.log_level {
            config.log.level = log_level;
        }

        config
    }

    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> Result<i32, Error> {
        match self.commands {
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

        let Config { server_endpoint, log } = self.load_config();
        log.registry();

        let fut = async move {
            let client = Client::new(server_endpoint).await?;

            match self.commands {
                None => {
                    print_list(&client, false).await?;
                }
                Some(Commands::List { no_id }) => {
                    print_list(&client, no_id).await?;
                }
                Some(Commands::Get { id }) => {
                    let data = if let Some(id) = id {
                        client.get(id).await?.printable_data(None)
                    } else {
                        client
                            .list(PREVIEW_LENGTH)
                            .await?
                            .into_iter()
                            .find(|metadata| metadata.kind == ClipboardKind::Clipboard)
                            .map(|metadata| metadata.preview)
                            .unwrap_or_default()
                    };

                    println!("{data}");
                }
                Some(Commands::Insert { kind, data }) => {
                    let _id = client.insert(data.as_bytes(), mime::TEXT_PLAIN_UTF_8, kind).await?;
                }
                Some(Commands::Length) => {
                    println!("{len}", len = client.length().await?);
                }
                Some(Commands::Load { kind, file_path, mime }) => {
                    let (data, mime) = load_file_or_read_stdin(file_path, mime).await?;
                    let _id = client.insert(&data, mime, kind).await?;
                }
                Some(Commands::Save { file_path, kind }) => {
                    let data = client.get_current_clip(kind).await?.encoded()?;
                    save_file_or_write_stdout(file_path, data).await?;
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
                            "{id:016x} is a {}, you could not edit with text editor",
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
                Some(Commands::Mark { id, mut kinds }) => {
                    if kinds.is_empty() {
                        kinds.push(ClipboardKind::Clipboard);
                    } else {
                        kinds.sort_unstable();
                        kinds.dedup();
                    }
                    for kind in kinds {
                        if client.mark(id, kind).await? {
                            println!("Ok ({kind})");
                        }
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
    let metadata_list = client.list(PREVIEW_LENGTH).await?;
    for metadata in metadata_list {
        let ClipEntryMetadata { id, preview, .. } = metadata;
        if no_id {
            println!("{preview}");
        } else {
            println!("{id:016x}: {preview}");
        }
    }
    Ok(())
}

async fn load_file_or_read_stdin(
    file_path: Option<PathBuf>,
    mime: mime::Mime,
) -> Result<(bytes::BytesMut, mime::Mime), Error> {
    let mut content = bytes::BytesMut::new();

    if let Some(file_path) = file_path {
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .await
            .context(error::ReadFileSnafu { filename: file_path.clone() })?;
        loop {
            let size = file
                .read_buf(&mut content)
                .await
                .context(error::ReadFileSnafu { filename: file_path.clone() })?;
            if size == 0 {
                break;
            }
        }
    } else {
        let mut file = tokio::io::stdin();
        loop {
            let size = file.read_buf(&mut content).await.context(error::ReadStdinSnafu)?;
            if size == 0 {
                break;
            }
        }
    }

    if mime.type_() == mime::TEXT {
        let _unused = simdutf8::basic::from_utf8(&content).context(error::CheckUtf8StringSnafu)?;
    }

    Ok((content, mime))
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

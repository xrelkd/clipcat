use std::{num::ParseIntError, path::PathBuf};

use snafu::ResultExt;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use clipcat::{editor::ExternalEditor, grpc::GrpcClient, ClipboardMode, ClipboardWatcherState};

use crate::{
    config::Config,
    error::{self, Error},
};

#[derive(StructOpt)]
#[structopt(name = clipcat::CTL_PROGRAM_NAME)]
pub struct Command {
    #[structopt(subcommand)]
    subcommand: Option<SubCommand>,

    #[structopt(long = "config", short = "c", help = "Specifies a configuration file")]
    config_file: Option<PathBuf>,

    #[structopt(short = "h", long = "host", help = "Specifies a server host")]
    server_host: Option<std::net::IpAddr>,

    #[structopt(short = "p", long = "port", help = "Specifies a server port")]
    server_port: Option<u16>,

    #[structopt(long = "log-level", help = "Specifies a log level")]
    log_level: Option<tracing::Level>,
}

#[derive(StructOpt)]
pub enum SubCommand {
    #[structopt(about = "Prints version information")]
    Version,

    #[structopt(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: structopt::clap::Shell },

    #[structopt(about = "Outputs default configuration")]
    DefaultConfig,

    #[structopt(aliases = &["paste"], about = "Insert new clip into clipboard")]
    Insert {
        #[structopt(
            short = "m",
            long = "mode",
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"selection\")"
        )]
        mode: ClipboardMode,

        data: String,
    },

    #[structopt(aliases = &["cut"], about = "Loads file into clipboard")]
    Load {
        #[structopt(
            short = "m",
            long = "mode",
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"selection\")"
        )]
        mode: ClipboardMode,

        #[structopt(long = "file", short = "f")]
        file_path: Option<PathBuf>,
    },

    #[structopt(aliases = &["paste"], about = "Pastes content of current clipboard into file")]
    Save {
        #[structopt(
            short = "m",
            long = "mode",
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"selection\")"
        )]
        mode: ClipboardMode,

        #[structopt(long = "file", short = "f")]
        file_path: Option<PathBuf>,
    },

    #[structopt(about = "Prints clip with <id>")]
    Get {
        #[structopt(parse(try_from_str = parse_hex))]
        id: Option<u64>,
    },

    #[structopt(
        aliases = &["ls"],
        about = "Prints history of clipboard")]
    List {
        #[structopt(long)]
        no_id: bool,
    },

    #[structopt(about = "Updates clip with <id>")]
    Update {
        #[structopt(parse(try_from_str = parse_hex))]
        id: u64,
        data: String,
    },

    #[structopt(about = "Edits clip with <id>")]
    Edit {
        #[structopt(env = "EDITOR", long = "editor", short = "e")]
        editor: String,

        #[structopt(parse(try_from_str = parse_hex))]
        id: u64,
    },

    #[structopt(
        aliases = &["rm", "delete", "del"],
        about = "Removes clips with [ids]")]
    Remove { ids: Vec<String> },

    #[structopt(name = "promote", about = "Replaces content of clipboard with clip with <id>")]
    Mark {
        #[structopt(
            short = "m",
            long = "mode",
            default_value = "clipboard",
            help = "Specifies which clipboard to insert (\"clipboard\", \"selection\")"
        )]
        mode: ClipboardMode,

        #[structopt(parse(try_from_str = parse_hex))]
        id: u64,
    },

    #[structopt(
        aliases = &["remove-all"],
        about = "Removes all clips in clipboard"
    )]
    Clear,

    #[structopt(
        aliases = &["count", "len"],
        about = "Prints length of clipboard history")]
    Length,

    #[structopt(aliases = &["enable"], about = "Enable clipboard watcher")]
    EnableWatcher,

    #[structopt(aliases = &["disable"], about = "Disable clipboard watcher")]
    DisableWatcher,

    #[structopt(aliases = &["toggle"], about = "Toggle clipboard watcher")]
    ToggleWatcher,

    #[structopt(aliases = &["watcher-state"], about = "Get clipboard watcher state")]
    GetWatcherState,
}

impl Command {
    pub fn new() -> Command { StructOpt::from_args() }

    fn load_config(&self) -> Config {
        let mut config =
            Config::load_or_default(&self.config_file.clone().unwrap_or_else(Config::default_path));
        if let Some(host) = self.server_host {
            config.server_host = host;
        }

        if let Some(port) = self.server_port {
            config.server_port = port;
        }

        if let Ok(log_level) = std::env::var("RUST_LOG") {
            use std::str::FromStr;
            config.log_level = tracing::Level::from_str(&log_level).unwrap_or(tracing::Level::INFO);
        }

        if let Some(log_level) = self.log_level {
            config.log_level = log_level;
        }

        config
    }

    pub fn run(self) -> Result<i32, Error> {
        fn print_watcher_state(state: ClipboardWatcherState) {
            let msg = match state {
                ClipboardWatcherState::Enabled => "Clipboard watcher is running",
                ClipboardWatcherState::Disabled => "Clipboard watcher is not running",
            };
            println!("{}", msg);
        }

        match self.subcommand {
            Some(SubCommand::Version) => {
                Self::clap()
                    .write_long_version(&mut std::io::stdout())
                    .expect("Failed to write to stdout");
                return Ok(0);
            }
            Some(SubCommand::Completions { shell }) => {
                Self::clap().gen_completions_to(
                    clipcat::CTL_PROGRAM_NAME,
                    shell,
                    &mut std::io::stdout(),
                );
                return Ok(0);
            }
            Some(SubCommand::DefaultConfig) => {
                use std::io::Write;
                let config_text =
                    toml::to_string_pretty(&Config::default()).expect("Config is serializable");
                std::io::stdout()
                    .write_all(&config_text.as_bytes())
                    .expect("Failed to write to stdout");
                return Ok(0);
            }
            _ => {}
        }

        let fut = async move {
            let config = self.load_config();

            {
                use tracing_subscriber::prelude::*;

                let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
                let level_filter =
                    tracing_subscriber::filter::LevelFilter::from_level(config.log_level);

                let registry = tracing_subscriber::registry().with(level_filter).with(fmt_layer);
                match tracing_journald::layer() {
                    Ok(layer) => registry.with(layer).init(),
                    Err(_err) => {
                        registry.init();
                    }
                }
            }

            let mut client =
                GrpcClient::new(format!("http://{}:{}", config.server_host, config.server_port))
                    .await?;

            match self.subcommand {
                None => {
                    print_list(&mut client, false).await?;
                }
                Some(SubCommand::List { no_id }) => {
                    print_list(&mut client, no_id).await?;
                }
                Some(SubCommand::Get { id }) => {
                    let data = match id {
                        Some(id) => client.get(id).await?,
                        None => {
                            let clips = client.list().await?;
                            clips
                                .into_iter()
                                .find(|entry| entry.mode == ClipboardMode::Clipboard)
                                .unwrap_or_default()
                        }
                    };
                    if data.is_utf8_string() {
                        println!("{}", data.as_utf8_string());
                    } else {
                        println!("{}", data.printable_data(None));
                    }
                }
                Some(SubCommand::Insert { mode, data }) => {
                    client.insert(data.as_bytes(), mime::TEXT_PLAIN_UTF_8, mode).await?;
                }
                Some(SubCommand::Length) => {
                    let len = client.length().await?;
                    println!("{}", len);
                }
                Some(SubCommand::Load { file_path, mode }) => {
                    let data = load_file_or_read_stdin(file_path).await?;
                    client.insert(&data.as_bytes(), mime::TEXT_PLAIN_UTF_8, mode).await?;
                }
                Some(SubCommand::Save { file_path, mode }) => {
                    let data = client.get_current_clip(mode).await?;
                    save_file_or_write_stdout(file_path, data.as_bytes()).await?;
                }
                Some(SubCommand::Remove { ids }) => {
                    let ids: Vec<u64> = ids
                        .into_iter()
                        .filter_map(|id| match parse_hex(&id) {
                            Ok(id) => Some(id),
                            Err(err) => {
                                eprintln!("Failed to parse ID {}, error: {:?}", id, err);
                                None
                            }
                        })
                        .collect();

                    if ids.is_empty() {
                        println!("Nothing is removed");
                        return Ok(0);
                    }
                    client.batch_remove(&ids).await?;
                }
                Some(SubCommand::Clear) => {
                    client.clear().await?;
                }
                Some(SubCommand::Edit { id, editor }) => {
                    let data = client.get(id).await?;
                    if data.is_utf8_string() {
                        let editor = ExternalEditor::new(editor);
                        let data = editor
                            .execute(&data.as_utf8_string())
                            .await
                            .context(error::CallEditor)?;
                        let (ok, new_id) =
                            client.update(id, data.as_bytes(), mime::TEXT_PLAIN_UTF_8).await?;
                        if ok {
                            println!("{:016x}", new_id);
                        }
                        client.mark(new_id, ClipboardMode::Clipboard).await?;
                    } else {
                        println!(
                            "{:016x} is a {}, you could not edit with text editor",
                            id,
                            data.mime_str()
                        );
                    }
                }
                Some(SubCommand::Update { id, data }) => {
                    let (ok, new_id) =
                        client.update(id, data.as_bytes(), mime::TEXT_PLAIN_UTF_8).await?;
                    if ok {
                        println!("{:016x}", new_id);
                    }
                }
                Some(SubCommand::Mark { id, mode }) => {
                    if client.mark(id, mode).await? {
                        println!("Ok");
                    }
                }
                Some(SubCommand::EnableWatcher) => {
                    let state = client.enable_watcher().await?;
                    print_watcher_state(state);
                }
                Some(SubCommand::DisableWatcher) => {
                    let state = client.disable_watcher().await?;
                    print_watcher_state(state);
                }
                Some(SubCommand::ToggleWatcher) => {
                    let state = client.toggle_watcher().await?;
                    print_watcher_state(state);
                }
                Some(SubCommand::GetWatcherState) => {
                    let state = client.get_watcher_state().await?;
                    print_watcher_state(state);
                }
                _ => unreachable!(),
            }
            Ok(0)
        };

        let runtime = Runtime::new().context(error::CreateTokioRuntime)?;
        runtime.block_on(fut)
    }
}

#[inline]
fn parse_hex(src: &str) -> Result<u64, ParseIntError> { u64::from_str_radix(src, 16) }

async fn print_list(client: &mut GrpcClient, no_id: bool) -> Result<(), Error> {
    const LINE_LENGTH: Option<usize> = Some(100);

    let list = client.list().await?;
    for data in list {
        if no_id {
            println!("{}", data.printable_data(LINE_LENGTH));
        } else {
            println!("{:016x}: {}", data.id, data.printable_data(LINE_LENGTH));
        }
    }
    Ok(())
}

async fn load_file_or_read_stdin(file_path: Option<PathBuf>) -> Result<String, Error> {
    use tokio::io::AsyncReadExt;
    match file_path {
        Some(file_path) => tokio::fs::read_to_string(&file_path)
            .await
            .context(error::ReadFile { filename: file_path.to_owned() }),
        None => {
            let mut data = String::new();
            tokio::io::stdin().read_to_string(&mut data).await.context(error::ReadStdin)?;
            Ok(data)
        }
    }
}

async fn save_file_or_write_stdout<C: AsRef<[u8]> + Unpin>(
    file_path: Option<PathBuf>,
    data: C,
) -> Result<(), Error> {
    use tokio::io::AsyncWriteExt;
    match file_path {
        Some(file_path) => tokio::fs::write(&file_path, data)
            .await
            .context(error::ReadFile { filename: file_path.to_owned() }),
        None => tokio::io::stdout().write_all(data.as_ref()).await.context(error::WriteStdout),
    }
}

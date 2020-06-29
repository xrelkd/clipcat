use std::{num::ParseIntError, path::PathBuf};

use snafu::ResultExt;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use clipcat::{editor::ExternalEditor, grpc::GrpcClient};

use crate::{
    config::Config,
    error::{self, Error},
};

#[derive(StructOpt)]
#[structopt(name = clipcat::CTL_PROGRAM_NAME)]
pub struct Command {
    #[structopt(subcommand)]
    subcommand: SubCommand,

    #[structopt(long = "config", short = "c", help = "Specifies a configuration file")]
    config_file: Option<PathBuf>,

    #[structopt(short = "h", long = "host", help = "Specifies a server host")]
    server_host: Option<std::net::IpAddr>,

    #[structopt(short = "p", long = "port", help = "Specifies a server port")]
    server_port: Option<u16>,

    #[structopt(long = "log-level", help = "Specifies a log level")]
    log_level: Option<log::LevelFilter>,
}

#[derive(StructOpt)]
pub enum SubCommand {
    #[structopt(about = "Prints version information")]
    Version,

    #[structopt(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: structopt::clap::Shell },

    #[structopt(about = "Outputs default configuration")]
    DefaultConfig,

    #[structopt(
        aliases = &["paste"],
        about = "Inserts new clip into clipboard")]
    Insert { data: String },

    #[structopt(about = "Inserts new clip into primary clipboard")]
    InsertPrimary { data: String },

    #[structopt(aliases = &["cut"], about = "Loads file into clipboard")]
    Load {
        #[structopt(long = "file", short = "f")]
        file_path: PathBuf,
    },

    #[structopt(aliases = &["cut-primary"], about = "Loads file into primary clipboard")]
    LoadPrimary {
        #[structopt(long = "file", short = "f")]
        file_path: PathBuf,
    },

    #[structopt(about = "Prints clip with <id>")]
    Get {
        #[structopt(parse(try_from_str = parse_hex))]
        id: Option<u64>,
    },

    #[structopt(
        aliases = &["ls"],
        about = "Prints content of clipboard")]
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
    MarkAsClipboard {
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
            use log::LevelFilter;
            use std::str::FromStr;
            config.log_level = LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Info);
        }

        if let Some(log_level) = self.log_level {
            config.log_level = log_level;
        }

        config
    }

    pub fn run(self) -> Result<i32, Error> {
        match self.subcommand {
            SubCommand::Version => {
                Self::clap()
                    .write_long_version(&mut std::io::stdout())
                    .expect("Failed to write to stdout");
                return Ok(0);
            }
            SubCommand::Completions { shell } => {
                Self::clap().gen_completions_to(
                    clipcat::CTL_PROGRAM_NAME,
                    shell,
                    &mut std::io::stdout(),
                );
                return Ok(0);
            }
            SubCommand::DefaultConfig => {
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

            std::env::set_var("RUST_LOG", config.log_level.to_string());
            env_logger::init();

            let mut client =
                GrpcClient::new(format!("http://{}:{}", config.server_host, config.server_port))
                    .await?;

            match self.subcommand {
                SubCommand::Get { id } => {
                    let data: String = match id {
                        Some(id) => client.get(id).await?,
                        None => {
                            let clips = client.list().await?;
                            clips
                                .into_iter()
                                .find(|entry| {
                                    entry.clipboard_type == clipcat::ClipboardType::Clipboard
                                })
                                .map(|v| v.data)
                                .unwrap_or_default()
                        }
                    };
                    println!("{}", data);
                }
                SubCommand::Insert { data } => {
                    client.insert_clipboard(&data).await?;
                }
                SubCommand::InsertPrimary { data } => {
                    client.insert_primary(&data).await?;
                }
                SubCommand::Length => {
                    let len = client.length().await?;
                    println!("{}", len);
                }
                SubCommand::Load { file_path } => {
                    let data = tokio::fs::read_to_string(&file_path)
                        .await
                        .context(error::ReadFile { filename: file_path.to_owned() })?;
                    client.insert_clipboard(&data).await?;
                }
                SubCommand::LoadPrimary { file_path } => {
                    let data = tokio::fs::read_to_string(&file_path)
                        .await
                        .context(error::ReadFile { filename: file_path.to_owned() })?;
                    client.insert_primary(&data).await?;
                }
                SubCommand::List { no_id } => {
                    const LINE_LENGTH: Option<usize> = Some(100);

                    let list = client.list().await?;
                    for data in list {
                        if no_id {
                            println!("{}", data.printable_data(LINE_LENGTH));
                        } else {
                            println!("{:016x}: {}", data.id, data.printable_data(LINE_LENGTH));
                        }
                    }
                }
                SubCommand::Remove { ids } => {
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
                SubCommand::Clear => {
                    client.clear().await?;
                }
                SubCommand::Edit { id, editor } => {
                    let data = client.get(id).await?;
                    let editor = ExternalEditor::new(editor);
                    let data = editor.execute(&data).await.context(error::CallEditor)?;
                    let (ok, new_id) = client.update(id, &data).await?;
                    if ok {
                        println!("{:016x}", new_id);
                    }
                    client.mark_as_clipboard(new_id).await?;
                }
                SubCommand::Update { id, data } => {
                    let (ok, new_id) = client.update(id, &data).await?;
                    if ok {
                        println!("{:016x}", new_id);
                    }
                }
                SubCommand::MarkAsClipboard { id } => {
                    if client.mark_as_clipboard(id).await? {
                        println!("Ok");
                    }
                }
                _ => unreachable!(),
            }
            Ok(0)
        };

        let mut runtime = Runtime::new().context(error::CreateTokioRuntime)?;
        runtime.block_on(fut)
    }
}

#[inline]
fn parse_hex(src: &str) -> Result<u64, ParseIntError> { u64::from_str_radix(src, 16) }

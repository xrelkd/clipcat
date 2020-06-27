use std::path::PathBuf;

use snafu::ResultExt;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use clipcat::{editor::ExternalEditor, grpc::GrpcClient};

use crate::{
    config::Config,
    error::{self, Error},
    selector::{ExternalSelector, SelectionMode, SelectorRunner},
};

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = clipcat::MENU_PROGRAM_NAME)]
pub struct Command {
    #[structopt(subcommand)]
    subcommand: Option<SubCommand>,

    #[structopt(long = "config", short = "c", help = "Specifies a configuration file")]
    config_file: Option<PathBuf>,

    #[structopt(long, short = "s", help = "Specifies a external selector")]
    selector: Option<ExternalSelector>,

    #[structopt(long, short = "m", help = "Specifies the menu length of selector")]
    menu_length: Option<usize>,

    #[structopt(long, short = "l", help = "Specifies the length of a line showing on selector")]
    line_length: Option<usize>,
}

#[derive(Debug, Clone, StructOpt)]
pub enum SubCommand {
    #[structopt(about = "Prints version information")]
    Version,

    #[structopt(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: structopt::clap::Shell },

    #[structopt(about = "Prints available text selectors")]
    ListSelector,

    #[structopt(about = "Outputs default configuration")]
    DefaultConfig,

    #[structopt(about = "Insert selected clip into clipboard")]
    Insert,

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
            Some(SubCommand::ListSelector) => {
                println!("{}", ExternalSelector::Rofi.to_string());
                println!("{}", ExternalSelector::Dmenu.to_string());
                println!("{}", ExternalSelector::Fzf.to_string());
                println!("{}", ExternalSelector::Skim.to_string());
                println!("{}", ExternalSelector::Custom.to_string());
                return Ok(());
            }
            Some(SubCommand::DefaultConfig) => {
                println!(
                    "{}",
                    toml::to_string_pretty(&Config::default()).expect("Config is serializable")
                );
                return Ok(());
            }
            _ => {}
        }

        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
        env_logger::init();

        let mut config =
            Config::load_or_default(&self.config_file.unwrap_or(Config::default_path()));
        let selector_runner = {
            if let Some(selector) = self.selector {
                config.selector = selector;
            }
            let mut runner = SelectorRunner::from_config(&config)?;
            if let Some(line_length) = self.line_length {
                runner.set_line_length(line_length);
            }
            if let Some(menu_length) = self.menu_length {
                runner.set_menu_length(menu_length);
            }
            runner
        };

        let subcommand = self.subcommand;
        let fut = async move {
            let grpc_addr = format!("http://{}:{}", config.server_host, config.server_port);
            let mut client = GrpcClient::new(grpc_addr).await?;
            let clips = client.list().await?;

            let selection_mode = match subcommand {
                Some(SubCommand::Insert) | Some(SubCommand::Edit { .. }) | None => {
                    SelectionMode::Single
                }
                Some(SubCommand::Remove) => SelectionMode::Multiple,
                _ => unreachable!(),
            };

            let selected_indices = selector_runner.run(&clips, selection_mode).await?;
            if selected_indices.is_empty() {
                println!("Nothing is selected");
                return Ok(());
            }

            let ids: Vec<u64> = selected_indices.iter().map(|index| clips[*index].id).collect();

            let selected_index = *selected_indices.first().expect("selected_indices is not empty");
            let id = clips[selected_index].id;
            let selected_data = &clips[selected_index as usize];

            match subcommand {
                Some(SubCommand::Remove) => {
                    client.batch_remove(&ids).await?;
                }
                Some(SubCommand::Insert) | None => {
                    const LINE_LENGTH: usize = 100;
                    println!(
                        "index: {}, content: {:?}",
                        selected_index,
                        selected_data.printable_data(Some(LINE_LENGTH)),
                    );
                    client.mark_as_clipboard(id).await?;
                }
                Some(SubCommand::Edit { editor }) => {
                    let editor = ExternalEditor::new(editor);
                    let new_data =
                        editor.execute(&selected_data.data).await.context(error::CallEditor)?;
                    let (ok, new_id) = client.update(id, &new_data).await?;
                    if ok {
                        println!("{:016x}", new_id);
                    }
                    client.mark_as_clipboard(new_id).await?;
                }
                _ => unreachable!(),
            }

            Ok(())
        };

        let mut runtime = Runtime::new().context(error::CreateTokioRuntime)?;
        runtime.block_on(fut)
    }
}

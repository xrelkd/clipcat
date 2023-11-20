mod error;

use std::io::Write;

use clap::{CommandFactory, Parser, Subcommand};
use clipcat::ClipboardMode;
use snafu::ResultExt;
use tokio::runtime::Runtime;

use self::error::Error;

#[derive(Parser)]
#[clap(name = clipcat::NOTIFY_PROGRAM_NAME)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Option<Commands>,

    #[clap(long = "no-clipboard", help = "Does not watch clipboard")]
    no_clipboard: bool,

    #[clap(long = "no-primary", help = "Does not watch primary")]
    no_primary: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    #[clap(about = "Prints version information")]
    Version,

    #[clap(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },
}

impl Cli {
    fn run(self) -> Result<(), Error> {
        match self.subcommand {
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

                if !enable_clipboard && !enable_primary {
                    return Err(Error::MonitorNothing);
                }

                Runtime::new().context(error::InitializeTokioRuntimeSnafu)?.block_on(async {
                    let clipboard_driver = clipcat_server::clipboard_driver::new()
                        .context(error::InitializeClipboardDriverSnafu)?;
                    let mut subscriber = clipboard_driver.subscribe().unwrap();
                    while let Some(mode) = subscriber.next().await {
                        match mode {
                            ClipboardMode::Clipboard if enable_clipboard => return Ok(()),
                            ClipboardMode::Selection if enable_primary => return Ok(()),
                            _ => continue,
                        }
                    }

                    drop(clipboard_driver);

                    Err(Error::WaitForClipboardEvent)
                })?;

                Ok(())
            }
        }
    }
}

fn main() {
    if let Err(err) = Cli::parse().run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

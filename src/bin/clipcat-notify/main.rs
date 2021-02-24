use snafu::{ResultExt, Snafu};
use structopt::StructOpt;
use tokio::runtime::Runtime;

use clipcat::ClipboardMode;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not initialize tokio Runtime, error: {}", source))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not create ClipboardDriver, error: {}", source))]
    InitializeClipboardDriver { source: clipcat::ClipboardError },

    #[snafu(display("Could not wait for clipboard event"))]
    WaitForClipboardEvent,

    #[snafu(display("Nothing to be monitored"))]
    MonitorNothing,
}

#[derive(Debug, StructOpt)]
#[structopt(name = clipcat::NOTIFY_PROGRAM_NAME)]
struct Command {
    #[structopt(subcommand)]
    subcommand: Option<SubCommand>,

    #[structopt(long = "no-clipboard", help = "Does not watch clipboard")]
    no_clipboard: bool,

    #[structopt(long = "no-primary", help = "Does not watch primary")]
    no_primary: bool,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    #[structopt(about = "Prints version information")]
    Version,

    #[structopt(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: structopt::clap::Shell },
}

impl Command {
    fn run(self) -> Result<(), Error> {
        match self.subcommand {
            Some(SubCommand::Version) => {
                Self::clap()
                    .write_long_version(&mut std::io::stdout())
                    .expect("Failed to write to stdout");
                return Ok(());
            }
            Some(SubCommand::Completions { shell }) => {
                Self::clap().gen_completions_to(
                    clipcat::NOTIFY_PROGRAM_NAME,
                    shell,
                    &mut std::io::stdout(),
                );
                return Ok(());
            }
            None => {}
        }

        let enable_clipboard = !self.no_clipboard;
        let enable_primary = !self.no_primary;

        if !enable_clipboard && !enable_primary {
            return Err(Error::MonitorNothing);
        }

        let runtime = Runtime::new().context(InitializeTokioRuntime)?;
        runtime.block_on(async {
            let clipboard_driver = clipcat::driver::new().context(InitializeClipboardDriver)?;
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

fn main() {
    let cmd = Command::from_args();
    if let Err(err) = cmd.run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

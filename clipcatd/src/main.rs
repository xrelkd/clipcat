mod command;
mod config;
mod error;
mod pid_file;

use self::{command::Cli, error::CommandError};

fn main() {
    if let Err(err) = Cli::default().run() {
        eprintln!("Error: {err}");
        std::process::exit(err.exit_code());
    }
}

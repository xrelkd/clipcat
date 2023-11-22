mod command;
mod config;
mod error;
mod finder;

use self::command::Cli;

fn main() {
    if let Err(err) = Cli::new().run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

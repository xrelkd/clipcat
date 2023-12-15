mod cli;
mod config;
mod error;
mod finder;

use self::cli::Cli;

fn main() {
    if let Err(err) = Cli::default().run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

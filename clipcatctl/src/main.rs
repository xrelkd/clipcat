mod cli;
mod config;
mod error;

use self::cli::Cli;

fn main() {
    match Cli::default().run() {
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    }
}

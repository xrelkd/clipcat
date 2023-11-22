mod command;
mod config;
mod error;

use self::command::Cli;

fn main() {
    match Cli::new().run() {
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    }
}

#[macro_use]
extern crate serde;

mod command;
mod config;
mod error;
mod finder;

use self::command::Command;

fn main() {
    let command = Command::new();
    if let Err(err) = command.run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

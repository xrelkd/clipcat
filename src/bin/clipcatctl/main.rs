#![cfg(feature = "app")]

#[macro_use]
extern crate serde;

#[macro_use]
extern crate snafu;

mod command;
mod config;
mod error;

use self::command::Command;

fn main() {
    let command = Command::new();
    match command.run() {
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    }
}

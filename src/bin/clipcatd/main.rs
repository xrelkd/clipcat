#[macro_use]
extern crate log;

#[macro_use]
extern crate snafu;

#[macro_use]
extern crate serde;

use std::sync::atomic;

mod command;
mod config;
mod error;
mod history;
mod worker;

use self::command::Command;

pub static SHUTDOWN: atomic::AtomicBool = atomic::AtomicBool::new(false);

fn main() {
    let command = Command::new();
    if let Err(err) = command.run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

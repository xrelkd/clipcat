mod command;
mod config;
mod error;
mod pid_file;

use mimalloc::MiMalloc;

use self::{command::Cli, error::CommandError};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    if let Err(err) = Cli::default().run() {
        eprintln!("Error: {err}");
        std::process::exit(err.exit_code());
    }
}

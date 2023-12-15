mod cli;
mod config;
mod error;

use mimalloc::MiMalloc;

use self::cli::Cli;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

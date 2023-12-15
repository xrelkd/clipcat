mod cli;
mod config;
mod error;
mod finder;

use mimalloc::MiMalloc;

use self::cli::Cli;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    if let Err(err) = Cli::default().run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

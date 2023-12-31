mod command;
mod config;
mod error;
mod pid_file;
mod shadow {
    #![allow(clippy::needless_raw_string_hashes)]
    use shadow_rs::shadow;
    shadow!(build);

    pub use self::build::*;
}

use self::{command::Cli, error::CommandError};

fn main() {
    if let Err(err) = Cli::default().run() {
        eprintln!("Error: {err}");
        std::process::exit(err.exit_code());
    }
}

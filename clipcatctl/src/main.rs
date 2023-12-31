mod cli;
mod config;
mod error;
mod shadow {
    #![allow(clippy::needless_raw_string_hashes)]
    use shadow_rs::shadow;
    shadow!(build);

    pub use self::build::*;
}

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

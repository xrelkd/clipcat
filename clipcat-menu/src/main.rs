mod cli;
mod config;
mod error;
mod finder;
mod shadow {
    #![allow(clippy::needless_raw_string_hashes)]
    use shadow_rs::shadow;
    shadow!(build);

    pub use self::build::*;
}

use self::cli::Cli;

fn main() {
    if let Err(err) = Cli::default().run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

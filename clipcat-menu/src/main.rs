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
        let error_msg = clipcat_cli::error_helpers::format_error_with_help(
            &err,
            "clipcat-menu",
            "clipcat-menu.toml",
        );
        eprintln!("Error: {error_msg}");
        std::process::exit(1);
    }
}

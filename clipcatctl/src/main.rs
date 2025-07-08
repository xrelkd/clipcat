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
            let error_msg = clipcat_cli::error_helpers::format_error_with_help(
                &err,
                "clipcatctl",
                "clipcatctl.toml",
            );
            eprintln!("Error: {error_msg}");
            std::process::exit(1);
        }
    }
}

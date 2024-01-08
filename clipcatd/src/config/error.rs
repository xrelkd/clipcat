use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not open config from {}, error: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}, error: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },

    #[snafu(display("Could not resolve file path {}, error: {source}", file_path.display()))]
    ResolveFilePath { file_path: PathBuf, source: std::io::Error },
}

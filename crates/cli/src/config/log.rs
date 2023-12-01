use std::{fs::OpenOptions, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing_subscriber::{
    layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt, Layer,
};

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogConfig {
    #[serde(default = "LogConfig::default_file_path")]
    pub file_path: Option<PathBuf>,

    #[serde(default = "LogConfig::default_emit_journald")]
    pub emit_journald: bool,

    #[serde(default = "LogConfig::default_emit_stdout")]
    pub emit_stdout: bool,

    #[serde(default = "LogConfig::default_emit_stderr")]
    pub emit_stderr: bool,

    #[serde(default = "LogConfig::default_log_level")]
    #[serde_as(as = "DisplayFromStr")]
    pub level: tracing::Level,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            file_path: Self::default_file_path(),
            emit_journald: Self::default_emit_journald(),
            emit_stdout: Self::default_emit_stdout(),
            emit_stderr: Self::default_emit_stderr(),
            level: Self::default_log_level(),
        }
    }
}

impl LogConfig {
    #[inline]
    #[must_use]
    pub const fn default_log_level() -> tracing::Level { tracing::Level::INFO }

    #[inline]
    #[must_use]
    pub const fn default_file_path() -> Option<PathBuf> { None }

    #[inline]
    #[must_use]
    pub const fn default_emit_journald() -> bool { true }

    #[inline]
    #[must_use]
    pub const fn default_emit_stdout() -> bool { false }

    #[inline]
    #[must_use]
    pub const fn default_emit_stderr() -> bool { false }

    pub fn registry(&self) {
        let Self { emit_journald, file_path, emit_stdout, emit_stderr, level: log_level } = self;

        let filter_layer = tracing_subscriber::filter::LevelFilter::from_level(*log_level);

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(emit_journald.then(|| LogDriver::Journald.layer()))
            .with(file_path.clone().map(|path| LogDriver::File(path).layer()))
            .with(emit_stdout.then(|| LogDriver::Stdout.layer()))
            .with(emit_stderr.then(|| LogDriver::Stderr.layer()))
            .init();
    }
}

#[derive(Clone, Debug)]
enum LogDriver {
    Stdout,
    Stderr,
    Journald,
    File(PathBuf),
}

impl LogDriver {
    /// # Panics
    #[allow(clippy::type_repetition_in_bounds)]
    fn layer<S>(self) -> Box<dyn Layer<S> + Send + Sync + 'static>
    where
        S: tracing::Subscriber,
        for<'a> S: LookupSpan<'a>,
    {
        // Shared configuration regardless of where logs are output to.
        let fmt =
            tracing_subscriber::fmt::layer().pretty().with_thread_ids(true).with_thread_names(true);

        // Configure the writer based on the desired log target:
        match self {
            Self::Stdout => Box::new(fmt.with_writer(std::io::stdout)),
            Self::Stderr => Box::new(fmt.with_writer(std::io::stderr)),
            Self::File(path) => {
                let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(path)
                    .expect("failed to create log file");
                Box::new(fmt.with_writer(file))
            }
            Self::Journald => {
                Box::new(tracing_journald::layer().expect("failed to open journald socket"))
            }
        }
    }
}

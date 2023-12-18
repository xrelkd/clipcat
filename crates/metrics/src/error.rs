use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not setup metrics, error: {source}",))]
    SetupMetrics { source: prometheus::Error, backtrace: Backtrace },

    #[snafu(display("Error occurs while serving metrics server, error: {message}",))]
    ServeMetricsServer { message: String },
}

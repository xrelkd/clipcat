pub mod dbus;
pub mod grpc;

use async_trait::async_trait;
use clipcat_metrics::error;
use snafu::ResultExt;

#[derive(Clone, Debug)]
pub struct Metrics {
    registry: prometheus::Registry,
}

impl Metrics {
    pub fn new() -> Result<Self, clipcat_metrics::Error> {
        let registry = prometheus::Registry::new();

        // gRPC
        registry
            .register(Box::new(self::grpc::REQUESTS_TOTAL.clone()))
            .context(error::SetupMetricsSnafu)?;

        // D-Bus
        registry
            .register(Box::new(self::dbus::REQUESTS_TOTAL.clone()))
            .context(error::SetupMetricsSnafu)?;
        registry
            .register(Box::new(self::dbus::REQUEST_DURATION_SECONDS.clone()))
            .context(error::SetupMetricsSnafu)?;

        Ok(Self { registry })
    }
}

#[async_trait]
impl clipcat_metrics::Metrics for Metrics {
    async fn gather(&self) -> Vec<prometheus::proto::MetricFamily> { self.registry.gather() }
}

#[cfg(test)]
mod tests {
    use crate::metrics::Metrics;

    #[test]
    fn test_new() { drop(Metrics::new().unwrap()); }
}

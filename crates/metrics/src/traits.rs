pub trait Metrics: Clone + Send + Sync {
    fn gather(&self) -> Vec<prometheus::proto::MetricFamily>;
}

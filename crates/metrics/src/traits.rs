use async_trait::async_trait;

#[async_trait]
pub trait Metrics: Clone + Send + Sync {
    async fn gather(&self) -> Vec<prometheus::proto::MetricFamily>;
}

use once_cell::sync::Lazy;
use prometheus::IntCounter;

pub static REQUESTS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    IntCounter::new("grpc_requests_total", "Total number of request from gRPC")
        .expect("setup metrics")
});

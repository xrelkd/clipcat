use lazy_static::lazy_static;
// use prometheus::{Histogram, HistogramOpts, IntCounter};
use prometheus::IntCounter;

lazy_static! {
    pub static ref REQUESTS_TOTAL: IntCounter =
        IntCounter::new("grpc_requests_total", "Total number of request from gRPC")
            .expect("setup metrics");
    // pub static ref REQUEST_DURATION_SECONDS: Histogram = Histogram::with_opts(HistogramOpts::new(
    //     "grpc_request_duration_seconds",
    //     "Latencies of handling request with gRPC in seconds"
    // ))
    // .expect("setup metrics");
}

use lazy_static::lazy_static;
use prometheus::IntCounter;

lazy_static! {
    pub static ref REQUESTS_TOTAL: IntCounter =
        IntCounter::new("grpc_requests_total", "Total number of request from gRPC")
            .expect("setup metrics");
}

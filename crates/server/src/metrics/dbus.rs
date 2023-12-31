use lazy_static::lazy_static;
use prometheus::{Histogram, HistogramOpts, IntCounter};

lazy_static! {
    pub static ref REQUESTS_TOTAL: IntCounter =
        IntCounter::new("dbus_requests_total", "Total number of request from D-Bus")
            .expect("setup metrics");
    pub static ref REQUEST_DURATION_SECONDS: Histogram = Histogram::with_opts(HistogramOpts::new(
        "dbus_request_duration_seconds",
        "Latencies of handling request with D-Bus in seconds"
    ))
    .expect("setup metrics");
}

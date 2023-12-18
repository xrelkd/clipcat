mod manager;
mod system;
mod watcher;

pub use self::{manager::ManagerService, system::SystemService, watcher::WatcherService};
use crate::metrics;

pub fn interceptor(req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
    metrics::grpc::REQUESTS_TOTAL.inc();

    Ok(req)
}

use clipcat_proto as proto;
use lazy_static::lazy_static;
use tonic::{Request, Response, Status};

lazy_static! {
    static ref GET_SYSTEM_VERSION_RESPONSE: proto::GetSystemVersionResponse =
        proto::GetSystemVersionResponse {
            major: clipcat_base::PROJECT_SEMVER.major,
            minor: clipcat_base::PROJECT_SEMVER.minor,
            patch: clipcat_base::PROJECT_SEMVER.patch
        };
}

pub struct SystemService {}

impl SystemService {
    #[inline]
    pub const fn new() -> Self { Self {} }
}

#[tonic::async_trait]
impl proto::System for SystemService {
    async fn get_version(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::GetSystemVersionResponse>, Status> {
        Ok(Response::new(GET_SYSTEM_VERSION_RESPONSE.clone()))
    }
}

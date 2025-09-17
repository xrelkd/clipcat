use async_trait::async_trait;
use clipcat_proto as proto;
use tonic::Request;

use crate::{error::GetSystemVersionError, Client};

#[async_trait]
pub trait System {
    async fn get_version(&self) -> Result<semver::Version, GetSystemVersionError>;
}

#[async_trait]
impl System for Client {
    async fn get_version(&self) -> Result<semver::Version, GetSystemVersionError> {
        let proto::GetSystemVersionResponse { major, minor, patch } =
            proto::SystemClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .max_decoding_message_size(self.max_decoding_message_size)
                .get_version(Request::new(()))
                .await
                .map_err(|source| GetSystemVersionError::Status { source })?
                .into_inner();
        Ok(semver::Version {
            major,
            minor,
            patch,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        })
    }
}

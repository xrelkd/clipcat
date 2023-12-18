use std::fmt;

use tonic::{metadata::AsciiMetadataValue, Request, Status};

use crate::metrics;

#[derive(Clone, Debug, Default)]
pub struct Interceptor {
    authorization_metadata_value: Option<AsciiMetadataValue>,
}

impl Interceptor {
    pub fn new<S>(access_token: Option<S>) -> Self
    where
        S: fmt::Display,
    {
        let authorization_metadata_value = match access_token {
            Some(token) if token.to_string().is_empty() => None,
            Some(token) => AsciiMetadataValue::try_from(format!("Bearer {token}"))
                .map_err(|err| {
                    tracing::warn!("{err}");
                })
                .ok(),
            None => None,
        };

        Self { authorization_metadata_value }
    }
}

impl tonic::service::Interceptor for Interceptor {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, Status> {
        metrics::grpc::REQUESTS_TOTAL.inc();

        if let Some(ref expected) = self.authorization_metadata_value {
            match req.metadata().get("authorization") {
                Some(token) if expected == token => Ok(req),
                _ => Err(Status::unauthenticated("No valid authorization token")),
            }
        } else {
            Ok(req)
        }
    }
}

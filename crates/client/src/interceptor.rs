use std::{fmt, sync::Arc};

use tonic::{metadata::AsciiMetadataValue, Request, Status};

#[derive(Clone, Debug, Default)]
pub struct Interceptor {
    authorization_metadata_value: Arc<Option<AsciiMetadataValue>>,
}

impl Interceptor {
    pub fn new<S>(access_token: Option<S>) -> Self
    where
        S: fmt::Display + Send,
    {
        let authorization_metadata_value = match access_token {
            Some(token) if token.to_string().is_empty() => None,
            Some(token) => AsciiMetadataValue::try_from(format!("Bearer {token}")).ok(),
            None => None,
        };

        Self { authorization_metadata_value: Arc::new(authorization_metadata_value) }
    }
}

impl tonic::service::Interceptor for Interceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        if let Some(ref token) = self.authorization_metadata_value.as_ref() {
            drop(req.metadata_mut().insert("authorization", token.clone()));
        }
        Ok(req)
    }
}

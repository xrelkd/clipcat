use std::{future::Future, net::SocketAddr, str::FromStr};

use axum::{
    body::Body,
    extract::Extension,
    http::{header, HeaderValue},
    response::Response,
    routing, Router,
};
use bytes::{BufMut, BytesMut};
use mime::Mime;
use once_cell::sync::Lazy;
use prometheus::{Encoder, TextEncoder};
use snafu::ResultExt;
use tokio::net::TcpListener;

use crate::{
    error::{self, Error},
    traits,
};

// FIXME: use `OPENMETRICS_TEXT`
#[allow(dead_code)]
static OPENMETRICS_TEXT: Lazy<Mime> = Lazy::new(|| {
    Mime::from_str("application/openmetrics-text; version=1.0.0; charset=utf-8")
        .expect("is valid mime type; qed")
});
static ENCODER: Lazy<TextEncoder> = Lazy::new(TextEncoder::new);

async fn metrics<Metrics>(Extension(metrics): Extension<Metrics>) -> Response<Body>
where
    Metrics: traits::Metrics + 'static,
{
    let mut buffer = BytesMut::new().writer();
    ENCODER
        .encode(&metrics.gather(), &mut buffer)
        .expect("`Writer<BytesMut>` should not encounter io error; qed");

    let mut res = Response::new(Body::from(buffer.into_inner().freeze()));
    drop(
        res.headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(ENCODER.format_type())),
    );
    res
}

fn metrics_index<Metrics>(m: Metrics) -> Router
where
    Metrics: traits::Metrics + 'static,
{
    Router::new().route("/metrics", routing::get(metrics::<Metrics>)).layer(Extension(m))
}

/// # Errors
///
/// * if it cannot bind server
pub async fn start_metrics_server<Metrics, ShutdownSignal>(
    listen_address: SocketAddr,
    metrics: Metrics,
    shutdown_signal: ShutdownSignal,
) -> Result<(), Error>
where
    Metrics: Clone + traits::Metrics + Send + 'static,
    ShutdownSignal: Future<Output = ()> + Send + 'static,
{
    let middleware_stack = tower::ServiceBuilder::new();

    let router = Router::new()
        .merge(metrics_index(metrics))
        .layer(middleware_stack)
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener =
        TcpListener::bind(&listen_address).await.context(error::BindMetricsServerSnafu)?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(|err| Error::ServeMetricsServer { message: err.to_string() })
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use crate::server::{ENCODER, OPENMETRICS_TEXT};

    #[test]
    fn test_once_cell_lazy() {
        let _ = Lazy::force(&OPENMETRICS_TEXT);
        let _ = Lazy::force(&ENCODER);
    }

    #[test]
    fn test_openmetrics_text_content_type() {
        assert_eq!(OPENMETRICS_TEXT.type_(), "application");
        assert_eq!(OPENMETRICS_TEXT.subtype(), "openmetrics-text");
        assert!(OPENMETRICS_TEXT.suffix().is_none());
        assert_eq!(OPENMETRICS_TEXT.get_param("charset").unwrap(), "utf-8");
        assert_eq!(OPENMETRICS_TEXT.get_param("version").unwrap(), "1.0.0");
    }
}

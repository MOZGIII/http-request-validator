//! The tests for both layer and validation logic.

#[derive(Debug, Clone)]
struct SampleValidator;

impl<Data: bytes::Buf + Sync> http_request_validator::Validator<Data> for SampleValidator {
    type Error = String;

    async fn validate<'a>(
        &'a self,
        _parts: &'a axum::http::request::Parts,
        _buffered_body: &'a Data,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn validate() {
    let result = super::validate(
        SampleValidator,
        axum::http::Request::new(axum::body::Body::empty()),
    )
    .await;

    let _ = result.unwrap();
}

#[tokio::test]
async fn validate_arc() {
    let result = super::validate(
        std::sync::Arc::new(SampleValidator),
        axum::http::Request::new(axum::body::Body::empty()),
    )
    .await;

    let _ = result.unwrap();
}

#[test]
fn layer_builds() {
    let _app: axum::Router<()> = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .route_layer(super::layer::new(SampleValidator));
}

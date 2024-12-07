//! The tests for tower layer and service.

use axum::response::IntoResponse as _;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SampleValidator;

impl<Data: bytes::Buf + Sync> http_request_validator::Validator<Data> for SampleValidator {
    type Error = ();

    async fn validate<'a>(
        &'a self,
        _parts: &'a axum::http::request::Parts,
        _buffered_body: &'a Data,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct MockInnerService;

impl tower_service::Service<axum::http::Request<axum::body::Body>> for MockInnerService {
    type Response = axum::response::Response;
    type Error = core::convert::Infallible;
    type Future = core::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: axum::http::Request<axum::body::Body>) -> Self::Future {
        core::future::ready(Ok("mock".into_response()))
    }
}

static_assertions::assert_impl_all!(
  super::Service<MockInnerService, SampleValidator, crate::axum::Bufferer, axum::body::Body, crate::axum::BufferedToBody>:
  tower_service::Service<axum::http::Request<axum::body::Body>, Error = crate::axum::Error<(), core::convert::Infallible>>,
);

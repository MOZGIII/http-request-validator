//! [`hyper`] integration for the [`http_request_validator`].

use hyper::{Body, Request};

/// The bufferer that uses [`hyper::body::to_bytes`] implementation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Bufferer;

#[async_trait::async_trait]
impl<Body> http_body_request_validator::Bufferer<Body> for Bufferer
where
    Body: http_body::Body + Send,
    for<'a> Body: 'a,
    <Body as http_body::Body>::Data: Send,
{
    type Data = bytes::Bytes;
    type Error = <Body as http_body::Body>::Error;

    async fn buffer(&self, body: Body) -> Result<Self::Data, Self::Error> {
        hyper::body::to_bytes(body).await
    }
}

/// The [`http_body_request_validator::Error`] type specialized for [`hyper`] [`Bufferer`].
pub type Error<V> = http_body_request_validator::Error<hyper::Error, V>;

/// Validate the [`hyper::Request`].
pub async fn validate<Validator>(
    validator: Validator,
    req: Request<Body>,
) -> Result<Request<Body>, Error<Validator::Error>>
where
    Validator: http_request_validator::Validator<bytes::Bytes>,
{
    http_body_request_validator::validate(Bufferer, validator, req).await
}

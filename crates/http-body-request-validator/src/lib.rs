//! A request validaton routine for generic [`http::Request`]s with an [`http_body::Body`].

use http::Request;

/// The ability to buffer an [`http_body::Body`].
#[async_trait::async_trait]
pub trait Bufferer<Body: http_body::Body> {
    /// The buffer type.
    type Data: bytes::Buf;
    /// An error that can occur while buffering.
    type Error;

    /// Buffer the given body into [`Self::Data`].
    async fn buffer(&self, body: Body) -> Result<Self::Data, Self::Error>;
}

/// An error that can occur while validating the request.
#[derive(Debug)]
pub enum Error<B, V> {
    /// The buffering of the request body failed.
    BodyBuffering(B),
    /// The validation failed.
    Validation(V),
}

/// Validate the given request.
pub async fn validate<InBody, OutBody, Bufferer, Validator>(
    bufferer: Bufferer,
    validator: Validator,
    req: Request<InBody>,
) -> Result<Request<OutBody>, Error<Bufferer::Error, Validator::Error>>
where
    InBody: http_body::Body,
    OutBody: http_body::Body,
    Bufferer: self::Bufferer<InBody>,
    OutBody: From<Bufferer::Data>,
    Validator: http_request_validator::Validator<Bufferer::Data>,
{
    let (parts, body) = req.into_parts();

    let buffered_body = bufferer.buffer(body).await.map_err(Error::BodyBuffering)?;

    validator
        .validate(&parts, &buffered_body)
        .await
        .map_err(Error::Validation)?;

    let req = Request::from_parts(parts, OutBody::from(buffered_body));

    Ok(req)
}

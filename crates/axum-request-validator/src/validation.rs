//! Validation logic for axum types.

use axum::body::Body;

/// The [`axum`]-specific validation error type.
pub type Error<V> = http_body_request_validator::Error<axum::Error, V>;

/// The bufferer and validator data type to use for axum.
pub type Data = axum::body::Bytes;

/// The bufferer type used by axum.
pub type Bufferer = http_body_request_validator::http_body_util::Bufferer<Data>;

/// The bufferer buffered type used by axum.
pub type BuffererBuffered = http_body_request_validator::bufferer::BufferedFor<Bufferer, Body>;

/// The custom implementation of the [`http_body_request_validator::convert::BufferedToBody`] for
/// axum [`Body`].
enum CustomBufferedToBody {}

impl http_body_request_validator::convert::BufferedToBody for CustomBufferedToBody {
    type Buffered = BuffererBuffered;
    type Body = Body;

    fn buffered_to_body(buffered: Self::Buffered) -> Self::Body {
        let body =
            http_body_request_validator::convert::Trivial::<Self::Buffered>::buffered_to_body(
                buffered,
            );
        Body::new(body)
    }
}

/// Validate the [`axum`] request.
pub async fn validate<Validator>(
    validator: Validator,
    req: axum::http::Request<Body>,
) -> Result<axum::http::Request<Body>, Error<Validator::Error>>
where
    Validator: http_request_validator::Validator<Data>,
{
    http_body_request_validator::BufferingValidator::new(Bufferer::new())
        .with_buffered_to_out_body::<CustomBufferedToBody>()
        .validate(validator, req)
        .await
}

//! [`axum`] integration for the [`http_request_validator`].

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

/// The [`axum`]-specific validation error type.
pub type Error<V> = http_body_request_validator::Error<axum::Error, V>;

/// The error handler for the validation errors.
#[axum::async_trait]
pub trait ErrorHandler<V> {
    /// Whatever the handler should respond with.
    type Response: IntoResponse;

    /// Handler the validation error.
    async fn handle_error(&self, error: Error<V>) -> Self::Response;
}

/// A an error renderer that will simply.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlainDisplayErrorRenderer;

#[axum::async_trait]
impl<V> ErrorHandler<V> for PlainDisplayErrorRenderer
where
    V: std::fmt::Display + Send,
    for<'a> V: 'a,
{
    type Response = (StatusCode, String);

    async fn handle_error(&self, error: Error<V>) -> Self::Response {
        match error {
            Error::BodyBuffering(error) => (
                StatusCode::BAD_REQUEST,
                format!("Unable to read the request: {error}"),
            ),
            Error::Validation(error) => (
                StatusCode::FORBIDDEN,
                format!("Invalid request signature: {error}"),
            ),
        }
    }
}

/// The bufferer data type used by axum.
pub type BuffererData = bytes::Bytes;

/// The bufferer type used by axum.
pub type Bufferer = http_body_request_validator::http_body_util::Bufferer<bytes::Bytes>;

/// The bufferer buffered type used by axum.
pub type BuffererBuffered = http_body_request_validator::bufferer::BufferedFor<Bufferer, Body>;

#[cfg(test)]
static_assertions::assert_type_eq_all!(BuffererData, bytes::Bytes);

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
    req: Request<Body>,
) -> Result<Request<Body>, Error<Validator::Error>>
where
    Validator: http_request_validator::Validator<BuffererData>,
{
    http_body_request_validator::BufferingValidator::new(Bufferer::new())
        .with_buffered_to_out_body::<CustomBufferedToBody>()
        .validate(validator, req)
        .await
}

/// [`axum`] middleware implementation.
///
/// Pass the error handler you need to handle the validation errors.
pub async fn middleware<Validator, ErrorHandler>(
    State((validator, error_handler)): State<(Validator, ErrorHandler)>,
    req: Request<Body>,
    next: Next,
) -> Result<
    axum::response::Response,
    <ErrorHandler as self::ErrorHandler<Validator::Error>>::Response,
>
where
    Validator: http_request_validator::Validator<BuffererData>,
    ErrorHandler: self::ErrorHandler<Validator::Error>,
{
    let req = match validate(validator, req).await {
        Ok(req) => req,
        Err(error) => return Err(error_handler.handle_error(error).await),
    };
    let res = next.run(req).await;
    Ok(res)
}

/// [`axum`] middleware implementation.
pub async fn simple_middleware<Validator>(
    State(validator): State<Validator>,
    req: Request<Body>,
    next: Next,
) -> Result<
    axum::response::Response,
    <PlainDisplayErrorRenderer as ErrorHandler<Validator::Error>>::Response,
>
where
    Validator: http_request_validator::Validator<BuffererData> + Send,
    <Validator as http_request_validator::Validator<BuffererData>>::Error:
        std::fmt::Display + Send + 'static,
{
    let error_handler = PlainDisplayErrorRenderer;
    middleware(State((validator, error_handler)), req, next).await
}

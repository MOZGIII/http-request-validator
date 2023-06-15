//! [`axum`] integration for the [`http_request_validator`].

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

pub use hyper_request_validator::{validate, Error};

/// The error handler for the validation errors.
#[async_trait::async_trait]
pub trait ErrorHandler<V> {
    /// Whatever the handler should respond with.
    type Response: IntoResponse;

    /// Handler the validation error.
    async fn handle_error(&self, error: Error<V>) -> Self::Response;
}

/// A an error renderer that wil simply.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlainDisplayErrorRenderer;

#[async_trait::async_trait]
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

/// [`axum`] middleware implementation.
///
/// Pass the error handler you need to handle the validation errors.
pub async fn middleware<Validator, ErrorHandler>(
    State((validator, error_handler)): State<(Validator, ErrorHandler)>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<
    axum::response::Response,
    <ErrorHandler as self::ErrorHandler<Validator::Error>>::Response,
>
where
    Validator: http_request_validator::Validator<bytes::Bytes>,
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
    next: Next<Body>,
) -> Result<
    axum::response::Response,
    <PlainDisplayErrorRenderer as ErrorHandler<Validator::Error>>::Response,
>
where
    Validator: http_request_validator::Validator<bytes::Bytes> + Send,
    <Validator as http_request_validator::Validator<bytes::Bytes>>::Error:
        std::fmt::Display + Send + 'static,
{
    let error_handler = PlainDisplayErrorRenderer;
    middleware(State((validator, error_handler)), req, next).await
}

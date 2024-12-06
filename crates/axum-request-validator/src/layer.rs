//! An axum layer for HTTP request validation.

use axum::{
    extract::Request,
    http,
    middleware::Next,
    response::{IntoResponse as _, Response},
};

use crate::Error;

/// A future that returns [`Response`].
pub type ResponseFuture =
    core::pin::Pin<Box<dyn core::future::Future<Output = Response> + Send + 'static>>;

/// The type alias for the fn used in the layer.
pub type Fn<S> = fn(axum::extract::State<S>, Request, Next) -> ResponseFuture;

/// The type alias for the extractors used in the layer.
pub type Extractors<S> = (axum::extract::State<S>, Request);

/// The layer type.
pub type Layer<State> = axum::middleware::FromFnLayer<Fn<State>, State, Extractors<State>>;

/// The layer state.
#[derive(Debug, Clone)]
pub struct State<Validator, ErrorHandler> {
    /// The validator to use.
    pub validator: Validator,

    /// The error handler to use.
    pub error_handler: ErrorHandler,
}

/// Create a new HTTP request validating layer.
///
/// ## Examples
///
/// ```
/// # #[derive(Clone)]
/// # struct MyValidator;
/// #
/// # impl<Data: bytes::Buf + Send + Sync> http_request_validator::Validator<Data> for MyValidator {
/// #    type Error = &'static str;
/// #
/// #    async fn validate<'a>(
/// #        &'a self,
/// #        _parts: &'a axum::http::request::Parts,
/// #        buffered_body: &'a Data,
/// #    ) -> Result<(), Self::Error> {
/// #        unimplemented!();
/// #    }
/// # }
/// #
/// use axum::{routing::get, Router};
///
/// let app = Router::new()
///     .route("/", get(|| async { "Hello, World!" }))
///     .route_layer(axum_request_validator::new(MyValidator));
/// # let _: Router<()> = app;
/// ```
pub fn new<Validator>(validator: Validator) -> Layer<State<Validator, PlainDisplayErrorRenderer>>
where
    Validator: http_request_validator::Validator<super::Data> + Send + 'static,
    <Validator as http_request_validator::Validator<super::Data>>::Error:
        std::fmt::Display + Send + Sync + 'static,
{
    with_error_handler(validator, PlainDisplayErrorRenderer)
}

/// Create a new HTTP request validating layer with custom error handling.
///
/// ## Examples
///
/// ```
/// # #[derive(Clone)]
/// # struct MyValidator;
/// #
/// # impl<Data: bytes::Buf + Send + Sync> http_request_validator::Validator<Data> for MyValidator {
/// #    type Error = &'static str;
/// #
/// #    async fn validate<'a>(
/// #        &'a self,
/// #        _parts: &'a axum::http::request::Parts,
/// #        buffered_body: &'a Data,
/// #    ) -> Result<(), Self::Error> {
/// #        unimplemented!();
/// #    }
/// # }
/// #
/// use axum::{routing::get, Router, http::StatusCode};
/// use axum_request_validator::{Error, ErrorHandler};
///
/// #[derive(Debug, Clone)]
/// struct MyErrorHandler;
///
/// impl<V> ErrorHandler<V> for MyErrorHandler
/// where
///     V: std::fmt::Display + Send + Sync + 'static,
/// {
///     type Response = (StatusCode, String);
///
///     async fn handle_error(&self, error: Error<V>) -> Self::Response {
///         match error {
///             Error::BodyBuffering(error) => (
///                 StatusCode::BAD_REQUEST,
///                 format!("Unable to buffer the request: {error}"),
///             ),
///             Error::Validation(error) => (
///                 StatusCode::FORBIDDEN,
///                 format!("Invalid request: {error}"),
///             ),
///         }
///     }
/// }
///
/// let app = Router::new()
///     .route("/", get(|| async { "Hello, World!" }))
///     .route_layer(axum_request_validator::with_error_handler(MyValidator, MyErrorHandler));
/// # let _: Router<()> = app;
/// ```
pub fn with_error_handler<Validator, ErrorHandler>(
    validator: Validator,
    error_handler: ErrorHandler,
) -> Layer<State<Validator, ErrorHandler>>
where
    Validator: http_request_validator::Validator<super::Data, Error: Send> + Send + 'static,
    ErrorHandler: self::ErrorHandler<Validator::Error> + Send + 'static,
{
    axum::middleware::from_fn_with_state(
        State {
            validator,
            error_handler,
        },
        |state, req, next| Box::pin(middleware(state, req, next)),
    )
}

/// The error handler for the validation errors.
pub trait ErrorHandler<V> {
    /// Whatever the handler should respond with.
    type Response: axum::response::IntoResponse;

    /// Handler the validation error.
    fn handle_error(
        &self,
        error: Error<V>,
    ) -> impl std::future::Future<Output = Self::Response> + Send + Sync;
}

/// A an error renderer that will simply.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlainDisplayErrorRenderer;

impl<V> ErrorHandler<V> for PlainDisplayErrorRenderer
where
    V: std::fmt::Display + Send + Sync,
    for<'a> V: 'a,
{
    type Response = (http::StatusCode, String);

    async fn handle_error(&self, error: Error<V>) -> Self::Response {
        match error {
            Error::BodyBuffering(error) => (
                http::StatusCode::BAD_REQUEST,
                format!("Unable to buffer the request: {error}"),
            ),
            Error::Validation(error) => (
                http::StatusCode::FORBIDDEN,
                format!("Invalid request: {error}"),
            ),
        }
    }
}

/// [`axum`] middleware-fn implementation.
pub fn middleware<Validator, ErrorHandler>(
    state: axum::extract::State<State<Validator, ErrorHandler>>,
    req: Request,
    next: Next,
) -> impl core::future::Future<Output = Response>
where
    Validator: http_request_validator::Validator<super::Data, Error: Send> + Send,
    ErrorHandler: self::ErrorHandler<Validator::Error> + Send,
{
    let axum::extract::State(State {
        validator,
        error_handler,
    }) = state;
    async move {
        let req = match super::validate(validator, req).await {
            Ok(req) => req,
            Err(error) => return error_handler.handle_error(error).await.into_response(),
        };
        next.run(req).await
    }
}

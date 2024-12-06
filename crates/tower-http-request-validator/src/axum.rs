//! Utility types for integration of the tower layers with [`axum`].

/// The [`axum`]-specific data type for bufferer and validators.
pub type Data = axum::body::Bytes;

/// The [`axum`]-specific bufferer.
pub type Bufferer = http_body_request_validator::http_body_util::Bufferer<Data>;

/// The converter for [`Bufferer::Buffered`] payload into [`axum::body::Body`].
pub struct BufferedToBody<InBody>(core::marker::PhantomData<fn() -> InBody>);

impl<InBody: http_body::Body<Data: Send> + Send>
    http_body_request_validator::convert::BufferedToBody for BufferedToBody<InBody>
{
    type Buffered = http_body_request_validator::bufferer::BufferedFor<Bufferer, InBody>;
    type Body = axum::body::Body;

    fn buffered_to_body(buffered: Self::Buffered) -> Self::Body {
        let body = http_body_request_validator::TrivialBufferedToOutBodyFor::<
            Bufferer,
            InBody,
        >::buffered_to_body(buffered);
        axum::body::Body::new(body)
    }
}

/// The [`axum`]-specific [`Layer`] type alias.
pub type Layer<Validator> =
    super::Layer<Validator, Bufferer, axum::body::Body, BufferedToBody<axum::body::Body>>;

impl<Validator> Layer<Validator> {
    /// Create a new [`axum`]-specific layer.
    pub fn for_axum(validator: Validator) -> Self {
        Self::new(validator, Bufferer::new())
    }
}

/// The [`axum`]-specific service error type.
pub type Error<ValidatorError, InnerError> = super::Error<
    http_body_request_validator::bufferer::ErrorFor<Bufferer, axum::body::Body>,
    ValidatorError,
    InnerError,
>;

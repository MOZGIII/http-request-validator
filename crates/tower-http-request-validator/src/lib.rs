//! An HTTP request validaton layer and service for tower.

#![no_std]

use core::marker::PhantomData;

extern crate alloc;

#[cfg(test)]
mod tests;

#[cfg(feature = "axum")]
pub mod axum;

/// An error that can occur in the service.
#[derive(Debug)]
pub enum Error<BodyBuffering, Validation, Inner = core::convert::Infallible> {
    /// Validation failed.
    Validation(http_body_request_validator::Error<BodyBuffering, Validation>),

    /// Inner service failed.
    Inner(Inner),
}

/// An HTTP request validation layer for tower.
#[derive(Debug)]
pub struct Layer<Validator, Bufferer, InBody, BufferedToOutBody> {
    /// The validator to use.
    pub validator: Validator,

    /// The bufferer to use.
    pub bufferer: Bufferer,

    /// The phantom data types.
    pub phantom_data: PhantomData<(fn() -> InBody, fn() -> BufferedToOutBody)>,
}

/// An HTTP request validation service for tower.
#[derive(Debug)]
pub struct Service<S, Validator, Bufferer, InBody, BufferedToOutBody> {
    /// The inner service.
    pub inner: S,

    /// The validator to use.
    pub validator: Validator,

    /// The bufferer to use.
    pub bufferer: Bufferer,

    /// The phantom data types.
    pub phantom_data: PhantomData<(fn() -> InBody, fn() -> BufferedToOutBody)>,
}

impl<Validator, Bufferer, InBody, BufferedToOutBody>
    Layer<Validator, Bufferer, InBody, BufferedToOutBody>
{
    /// Create a new HTTP request validation layer with the given settings.
    pub fn new(validator: Validator, bufferer: Bufferer) -> Self {
        Self {
            validator,
            bufferer,
            phantom_data: PhantomData,
        }
    }

    /// Alter the `BufferedToOutBody` type.
    pub fn with_buffered_to_out_body<Other>(self) -> Layer<Validator, Bufferer, InBody, Other> {
        let Self {
            validator,
            bufferer,
            phantom_data: PhantomData,
        } = self;
        Layer {
            validator,
            bufferer,
            phantom_data: PhantomData,
        }
    }
}

impl<Validator: Clone, Bufferer: Clone, InBody, BufferedToOutBody> Clone
    for Layer<Validator, Bufferer, InBody, BufferedToOutBody>
{
    fn clone(&self) -> Self {
        Self {
            validator: self.validator.clone(),
            bufferer: self.bufferer.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl<S: Clone, Bufferer: Clone, Validator: Clone, InBody, BufferedToOutBody> Clone
    for Service<S, Bufferer, Validator, InBody, BufferedToOutBody>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            validator: self.validator.clone(),
            bufferer: self.bufferer.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl<S, Bufferer: Clone, Validator: Clone, InBody, BufferedToOutBody> tower_layer::Layer<S>
    for Layer<Validator, Bufferer, InBody, BufferedToOutBody>
{
    type Service = Service<S, Validator, Bufferer, InBody, BufferedToOutBody>;

    fn layer(&self, inner: S) -> Self::Service {
        Service {
            inner,
            bufferer: self.bufferer.clone(),
            validator: self.validator.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl<S, Validator, Bufferer, InBody, BufferedToOutBody>
    tower_service::Service<http::Request<InBody>>
    for Service<S, Validator, Bufferer, InBody, BufferedToOutBody>
where
    InBody: http_body::Body + 'static,
    Bufferer: http_body_request_validator::Bufferer<InBody, Buffered: , Error: > + 'static,
    Validator: http_request_validator::Validator<
            http_body_request_validator::bufferer::DataFor<Bufferer, InBody>,
        > + 'static,
    BufferedToOutBody: http_body_request_validator::convert::BufferedToBody<Buffered = Bufferer::Buffered>
        + 'static,
    S: tower_service::Service<http::Request<BufferedToOutBody::Body>> + 'static,

    Bufferer: Clone,
    Validator: Clone,
    S: Clone,
{
    type Response = S::Response;
    type Error = Error<Bufferer::Error, Validator::Error, S::Error>;
    type Future = ResponseFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Error::Inner)
    }

    fn call(&mut self, req: http::Request<InBody>) -> Self::Future {
        alloc::boxed::Box::pin(self.call_unboxed(req))
    }
}

impl<S, Validator, Bufferer, InBody, BufferedToOutBody>
    Service<S, Validator, Bufferer, InBody, BufferedToOutBody>
where
    InBody: http_body::Body + 'static,
    Bufferer: http_body_request_validator::Bufferer<InBody, Buffered: , Error: > + 'static,
    Validator: http_request_validator::Validator<
            http_body_request_validator::bufferer::DataFor<Bufferer, InBody>,
        > + 'static,
    BufferedToOutBody: http_body_request_validator::convert::BufferedToBody<Buffered = Bufferer::Buffered>
        + 'static,
    S: tower_service::Service<http::Request<BufferedToOutBody::Body>> + 'static,

    Bufferer: Clone,
    Validator: Clone,
    S: Clone,
{
    #[allow(clippy::missing_docs_in_private_items, clippy::type_complexity)]
    fn call_unboxed(
        &mut self,
        req: http::Request<InBody>,
    ) -> impl core::future::Future<
        Output = Result<S::Response, Error<Bufferer::Error, Validator::Error, S::Error>>,
    > {
        let not_ready_inner = self.inner.clone();
        let ready_inner = core::mem::replace(&mut self.inner, not_ready_inner);

        let buffering_validator =
            http_body_request_validator::BufferingValidator::new(self.bufferer.clone())
                .with_buffered_to_out_body::<BufferedToOutBody>();
        let validator = self.validator.clone();

        Self::call_impl(ready_inner, buffering_validator, validator, req)
    }

    #[allow(clippy::missing_docs_in_private_items)]
    async fn call_impl(
        mut ready_inner: S,
        buffering_validator: http_body_request_validator::BufferingValidator<
            Bufferer,
            InBody,
            BufferedToOutBody,
        >,
        validator: Validator,
        req: http::Request<InBody>,
    ) -> Result<S::Response, Error<Bufferer::Error, Validator::Error, S::Error>> {
        let req = Self::validate_request(buffering_validator, validator, req)
            .await
            .map_err(Error::Validation)?;
        ready_inner.call(req).await.map_err(Error::Inner)
    }

    #[allow(clippy::missing_docs_in_private_items)]
    async fn validate_request(
        buffering_validator: http_body_request_validator::BufferingValidator<
            Bufferer,
            InBody,
            BufferedToOutBody,
        >,
        validator: Validator,
        req: http::Request<InBody>,
    ) -> Result<
        http::Request<BufferedToOutBody::Body>,
        http_body_request_validator::Error<Bufferer::Error, Validator::Error>,
    > {
        buffering_validator.validate(validator, req).await
    }
}

/// The response future.
pub type ResponseFuture<T> =
    core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = T> + Send + 'static>>;

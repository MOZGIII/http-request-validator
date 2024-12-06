//! [`BufferingValidator`] implementation and associated types.

use crate::AsBuf as _;

/// An error that can occur while validating the request.
#[derive(Debug)]
pub enum Error<B, V> {
    /// The buffering of the request body failed.
    BodyBuffering(B),
    /// The validation failed.
    Validation(V),
}

/// The trivial [`crate::convert::BufferedToBody`] implementation for a given bufferer.
pub type TrivialBufferedToOutBodyFor<Bufferer, InBody> =
    crate::convert::Trivial<crate::bufferer::BufferedFor<Bufferer, InBody>>;

/// The buffering validator.
pub struct BufferingValidator<
    Bufferer,
    InBody,
    BufferedToOutBody = TrivialBufferedToOutBodyFor<Bufferer, InBody>,
> {
    /// The bufferer.
    pub bufferer: Bufferer,

    /// The phantom data types.
    pub phantom_data: core::marker::PhantomData<(fn() -> InBody, BufferedToOutBody)>,
}

impl<Bufferer, InBody>
    BufferingValidator<Bufferer, InBody, TrivialBufferedToOutBodyFor<Bufferer, InBody>>
where
    InBody: http_body::Body,
    Bufferer: crate::Bufferer<InBody>,
{
    /// Create a new [`BufferingValidator`] with trivial `BufferedToOutBody`.
    pub const fn new(bufferer: Bufferer) -> Self {
        Self {
            bufferer,
            phantom_data: core::marker::PhantomData,
        }
    }
}

impl<Bufferer, InBody, BufferedToOutBody> BufferingValidator<Bufferer, InBody, BufferedToOutBody>
where
    InBody: http_body::Body,
    Bufferer: crate::Bufferer<InBody>,
{
    /// Change the `BufferedToOutBody` type.
    pub fn with_buffered_to_out_body<New>(self) -> BufferingValidator<Bufferer, InBody, New> {
        let Self {
            bufferer,
            phantom_data: _,
        } = self;
        BufferingValidator {
            bufferer,
            phantom_data: core::marker::PhantomData,
        }
    }
}

impl<Bufferer, InBody, BufferedToOutBody> BufferingValidator<Bufferer, InBody, BufferedToOutBody>
where
    InBody: http_body::Body,
    Bufferer: crate::Bufferer<InBody>,
    BufferedToOutBody:
        crate::convert::BufferedToBody<Buffered = crate::bufferer::BufferedFor<Bufferer, InBody>>,
{
    /// Validate the given request.
    ///
    /// Takes the `InBody` out of the request, buffers it, validates the buffered body data using
    /// the specified validator, and then converts the buffered stuff with `BufferedToOutBody` to
    /// get the `BufferedToOutBody::Body` type.
    pub async fn validate<Validator>(
        &self,
        validator: Validator,
        req: http::Request<InBody>,
    ) -> Result<http::Request<BufferedToOutBody::Body>, Error<Bufferer::Error, Validator::Error>>
    where
        Validator: http_request_validator::Validator<crate::bufferer::DataFor<Bufferer, InBody>>,
    {
        let (parts, body) = req.into_parts();

        let buffered = self
            .bufferer
            .buffer(body)
            .await
            .map_err(Error::BodyBuffering)?;

        validator
            .validate(&parts, buffered.as_buf())
            .await
            .map_err(Error::Validation)?;

        let req = http::Request::from_parts(parts, BufferedToOutBody::buffered_to_body(buffered));

        Ok(req)
    }
}

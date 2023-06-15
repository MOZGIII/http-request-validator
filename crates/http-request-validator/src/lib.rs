//! A generic HTTP request validator.

/// The [`http::Request`] validator.
///
/// Runs over the buffered request body, so can be used to implement the request signature
/// validation, or anything that needs a whole request available to conduct the validation.
///
/// You can provide your validation logic in this trait implementation.
/// See the neighbouring crates for integrations with various web servers.
#[async_trait::async_trait]
pub trait Validator<Data: bytes::Buf> {
    /// An error that can occur during validation.
    type Error;

    /// Validate the request header and buffered body.
    async fn validate(
        &self,
        parts: &http::request::Parts,
        buffered_body: &Data,
    ) -> Result<(), Self::Error>;
}

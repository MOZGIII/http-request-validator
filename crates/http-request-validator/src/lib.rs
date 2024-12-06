//! A generic HTTP request validator.

#![no_std]

/// The [`http::Request`] validator.
///
/// Runs over the buffered request body, so can be used to implement the request signature
/// validation, or anything that needs a whole request available to conduct the validation.
///
/// You can provide your validation logic in this trait implementation.
/// See the neighbouring crates for integrations with various web servers.
pub trait Validator<Data: bytes::Buf> {
    /// An error that can occur during validation.
    type Error;

    /// Validate the request header and buffered body.
    fn validate<'a>(
        &'a self,
        parts: &'a http::request::Parts,
        buffered_body: &'a Data,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> + Send + 'a;
}

impl<T: ?Sized, Data> Validator<Data> for T
where
    T: core::ops::Deref + Send + Sync,
    <T as core::ops::Deref>::Target: Validator<Data> + Send,
    Data: bytes::Buf + Send + Sync,
{
    type Error = <<T as core::ops::Deref>::Target as Validator<Data>>::Error;

    fn validate<'a>(
        &'a self,
        parts: &'a http::request::Parts,
        buffered_body: &'a Data,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> + 'a {
        self.deref().validate(parts, buffered_body)
    }
}

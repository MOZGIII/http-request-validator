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

#[async_trait::async_trait]
impl<T, Data> Validator<Data> for &T
where
    T: Validator<Data> + Send + Sync,
    Data: bytes::Buf + Send + Sync,
{
    type Error = T::Error;

    async fn validate(
        &self,
        parts: &http::request::Parts,
        buffered_body: &Data,
    ) -> Result<(), Self::Error> {
        <&T as std::ops::Deref>::deref(self)
            .validate(parts, buffered_body)
            .await
    }
}

#[async_trait::async_trait]
impl<T, Data> Validator<Data> for std::sync::Arc<T>
where
    T: Validator<Data> + Send + Sync,
    Data: bytes::Buf + Send + Sync,
{
    type Error = T::Error;

    async fn validate(
        &self,
        parts: &http::request::Parts,
        buffered_body: &Data,
    ) -> Result<(), Self::Error> {
        <std::sync::Arc<T> as std::ops::Deref>::deref(self)
            .validate(parts, buffered_body)
            .await
    }
}

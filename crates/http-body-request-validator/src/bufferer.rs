//! [`Bufferer`] trait and utils.

/// The ability to buffer and validate an [`http_body::Body`].
pub trait Bufferer<InBody: http_body::Body> {
    /// The buffered body type.
    type Buffered: crate::AsBuf;

    /// An error that can occur while buffering.
    type Error;

    /// Buffer the given body into [`Self::BufferedBody`].
    fn buffer(
        &self,
        body: InBody,
    ) -> impl core::future::Future<Output = Result<Self::Buffered, Self::Error>>;
}

/// Extract bufferer buffered type.
pub type BufferedFor<Bufferer, InBody> = <Bufferer as self::Bufferer<InBody>>::Buffered;

/// Extract bufferer data type.
pub type DataFor<Bufferer, InBody> = <BufferedFor<Bufferer, InBody> as crate::AsBuf>::Data;

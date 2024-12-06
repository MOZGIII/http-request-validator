//! Types for storing buffered body data.
//!
//! If the provided types don't fir the implementation for whatever reason it is easy to implement
//! your own.

/// Buffered body parts.
///
/// Intended to represent.
pub struct Buffered<Data: bytes::Buf> {
    /// The body data.
    ///
    /// Must be viewable as a [`bytes::Buf`].
    pub data: Data,

    /// The buffered trailers, if any.
    pub trailers: Option<http::HeaderMap>,
}

impl<Data: bytes::Buf> super::AsBuf for Buffered<Data> {
    type Data = Data;

    fn as_buf(&self) -> &Self::Data {
        &self.data
    }
}

pin_project_lite::pin_project! {
    /// Buffered body implementation.
    pub struct Body<Data: bytes::Buf> {
        data: Option<Data>,
        trailers: Option<http::HeaderMap>,
    }
}

impl<Data: bytes::Buf> Body<Data> {
    /// Create a buffered [`Body`] from [`Buffered`].
    pub fn from_buffered(value: Buffered<Data>) -> Self {
        Self {
            data: Some(value.data),
            trailers: value.trailers,
        }
    }
}

impl<Data: bytes::Buf> crate::convert::IntoBody for Buffered<Data> {
    type Body = Body<Data>;

    fn into_body(self) -> Self::Body {
        Body::from_buffered(self)
    }
}

impl<Data: bytes::Buf> http_body::Body for Body<Data> {
    type Data = Data;
    type Error = core::convert::Infallible;

    fn poll_frame(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.project();

        let frame = if let Some(data) = this.data.take() {
            http_body::Frame::data(data)
        } else if let Some(trailers) = this.trailers.take() {
            http_body::Frame::trailers(trailers)
        } else {
            return core::task::Poll::Ready(None);
        };

        core::task::Poll::Ready(Some(Ok(frame)))
    }
}

//! [`AsBuf`] trait.

/// The ability view something as a [`bytes::Buf`].
pub trait AsBuf {
    /// The data type that implements [`bytes::Buf`] as which we can view self.
    type Data: bytes::Buf;

    /// View as [`Self::Data`] that implements [`bytes::Buf`].
    fn as_buf(&self) -> &Self::Data;
}

impl<T: bytes::Buf> AsBuf for T {
    type Data = T;

    fn as_buf(&self) -> &Self::Data {
        self
    }
}

//! The [`http-body-util`]-powered bufferer.

/// The bufferer that uses [`http_body_util`] implementation that aggregates into [`bytes::Bytes`].
#[derive(Debug, Copy)]
pub struct Bufferer<Data>(core::marker::PhantomData<Data>);

impl<Data> Bufferer<Data> {
    /// Create a new [`Bufferer`] instance.
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<Data> Default for Bufferer<Data> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Data> Clone for Bufferer<Data> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<InBody> super::Bufferer<InBody> for Bufferer<bytes::Bytes>
where
    InBody: http_body::Body,
{
    type Buffered = crate::buffered::Buffered<bytes::Bytes>;
    type Error = <InBody as http_body::Body>::Error;

    async fn buffer(&self, body: InBody) -> Result<Self::Buffered, Self::Error> {
        let collected_body = http_body_util::BodyExt::collect(body).await?;
        let trailers = collected_body.trailers().cloned();
        let data = collected_body.to_bytes();
        Ok(crate::buffered::Buffered { data, trailers })
    }
}

/// The boxed [`bytes::Buf`].
#[cfg(feature = "alloc")]
pub type BoxBuf = alloc::boxed::Box<dyn bytes::Buf>;

#[cfg(feature = "alloc")]
impl<InBody> super::Bufferer<InBody> for Bufferer<BoxBuf>
where
    InBody: http_body::Body,
    <InBody as http_body::Body>::Data: 'static,
{
    type Buffered = crate::buffered::Buffered<BoxBuf>;
    type Error = <InBody as http_body::Body>::Error;

    async fn buffer(&self, body: InBody) -> Result<Self::Buffered, Self::Error> {
        let collected_body = http_body_util::BodyExt::collect(body).await?;
        let trailers = collected_body.trailers().cloned();
        let data = alloc::boxed::Box::new(collected_body.aggregate());
        Ok(crate::buffered::Buffered { data, trailers })
    }
}

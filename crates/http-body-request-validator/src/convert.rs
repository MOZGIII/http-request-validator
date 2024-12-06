//! Conversion traits and utils.

/// Conversion of a buffered type into a body.
pub trait BufferedToBody {
    /// The buffered body parts type to convert from.
    type Buffered;

    /// The body type to convert to.
    type Body: http_body::Body;

    /// Consume buffered and turn it into a body.
    fn buffered_to_body(buffered: Self::Buffered) -> Self::Body;
}

/// Convert self into body.
pub trait IntoBody {
    /// The body type to convert to.
    type Body: http_body::Body;

    /// Consume self and turn into a body.
    fn into_body(self) -> Self::Body;
}

/// The default [`BufferedToBody`] implementation that works for all [`IntoBody`] implementations.
///
/// Swap this type with your own to alter the type of body that the conversion targets.
///
/// This type in not intended to be instantiated.
pub struct Trivial<Buffered>(pub(crate) core::marker::PhantomData<Buffered>);

impl<Buffered> BufferedToBody for Trivial<Buffered>
where
    Buffered: IntoBody,
{
    type Buffered = Buffered;
    type Body = <Buffered as IntoBody>::Body;

    fn buffered_to_body(buffered: Buffered) -> Self::Body {
        buffered.into_body()
    }
}

//! A request validaton routine for generic [`http::Request`]s with an [`http_body::Body`].

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod as_buf;
pub mod bufferer;
pub mod buffering_validator;
pub mod convert;

#[cfg(feature = "buffered")]
pub mod buffered;

#[cfg(feature = "http-body-util")]
pub mod http_body_util;

pub use self::as_buf::AsBuf;
pub use self::bufferer::Bufferer;

pub use self::buffering_validator::*;

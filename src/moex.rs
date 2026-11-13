//! HTTP-клиент для ISS API и ошибки транспортного уровня.

#[cfg(any(feature = "async", feature = "blocking"))]
mod client;
mod constants;
pub mod decode;
mod endpoint;
mod error;
mod payload;
#[cfg(any(feature = "async", feature = "blocking"))]
mod request;
mod wire;

#[cfg(any(feature = "async", feature = "blocking"))]
pub use client::*;

pub use endpoint::IssEndpoint;
pub use error::MoexError;
#[cfg(any(feature = "async", feature = "blocking"))]
pub use request::*;

#[cfg(test)]
mod tests;

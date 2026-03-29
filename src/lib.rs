//! Типизированный Rust-клиент для работы с ISS API Московской биржи.

/// Асинхронный API транспортного слоя.
#[cfg(feature = "async")]
pub mod r#async;
/// Блокирующий API транспортного слоя.
#[cfg(feature = "blocking")]
pub mod blocking;
/// Транспортно-независимый API декодирования ISS JSON payload-ов.
///
/// Реэкспортируется из внутреннего модуля `moex::decode` и доступен как
/// `moex_client::decode`.
pub use crate::moex::decode;
/// Доменные типы и парсинг ответов ISS в строгие модели.
pub mod models;
mod moex;
/// Удобный импорт extension-traits для fluent-цепочек над коллекциями.
pub mod prelude;

pub use moex::{IssEndpoint, MoexError};
#[cfg(any(feature = "async", feature = "blocking"))]
pub use moex::{
    IssRequestOptions, IssToggle, RateLimit, RateLimiter, RawIssResponse, RetryPolicy,
    with_rate_limit, with_retry,
};
#[cfg(feature = "async")]
pub use moex::{with_rate_limit_async, with_retry_async};

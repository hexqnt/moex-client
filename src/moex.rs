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

/// Асинхронный ленивый пагинатор по страницам `history`.
#[cfg(all(feature = "async", feature = "history"))]
pub use client::AsyncHistoryPages;
/// Блокирующий ленивый пагинатор по страницам `history`.
#[cfg(all(feature = "blocking", feature = "history"))]
pub use client::HistoryPages;
/// Асинхронные HTTP-клиенты ISS API.
#[cfg(feature = "async")]
pub use client::{
    AsyncCandlesPages, AsyncGlobalSecuritiesPages, AsyncIndexAnalyticsPages,
    AsyncMarketSecuritiesPages, AsyncMarketTradesPages, AsyncMoexClient, AsyncMoexClientBuilder,
    AsyncOwnedBoardScope, AsyncOwnedEngineScope, AsyncOwnedIndexScope, AsyncOwnedMarketScope,
    AsyncOwnedMarketSecurityScope, AsyncOwnedSecurityResourceScope, AsyncOwnedSecurityScope,
    AsyncRawIssRequestBuilder, AsyncSecStatsPages, AsyncSecuritiesPages, AsyncTradesPages,
};
#[cfg(all(feature = "async", feature = "news"))]
/// Асинхронные пагинаторы новостных endpoint-ов.
pub use client::{AsyncEventsPages, AsyncSiteNewsPages};
/// Блокирующие HTTP-клиенты ISS API.
#[cfg(feature = "blocking")]
pub use client::{
    CandlesPages, GlobalSecuritiesPages, IndexAnalyticsPages, MarketSecuritiesPages,
    MarketTradesPages, OwnedBoardScope, OwnedEngineScope, OwnedIndexScope, OwnedMarketScope,
    OwnedMarketSecurityScope, OwnedSecurityResourceScope, OwnedSecurityScope, RawIssRequestBuilder,
    SecStatsPages, SecuritiesPages, TradesPages,
};
#[cfg(all(feature = "blocking", feature = "news"))]
/// Блокирующие пагинаторы новостных endpoint-ов.
pub use client::{EventsPages, SiteNewsPages};

pub use endpoint::IssEndpoint;
pub use error::MoexError;
#[cfg(any(feature = "async", feature = "blocking"))]
pub use request::*;

/// Явное имя блокирующего ISS-клиента.
#[cfg(feature = "blocking")]
pub type BlockingMoexClient = client::BlockingMoexClient;
/// Явное имя типа builder для блокирующего ISS-клиента.
#[cfg(feature = "blocking")]
pub type BlockingMoexClientBuilder = client::BlockingMoexClientBuilder;

#[cfg(test)]
mod tests;

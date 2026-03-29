//! Асинхронный API клиента ISS.

#[cfg(feature = "history")]
/// Ленивый пагинатор по `history` в асинхронном API.
pub use crate::moex::AsyncHistoryPages as HistoryPages;
/// Основные пагинаторы и типы контекстов асинхронного API.
pub use crate::moex::{
    AsyncCandlesPages as CandlesPages, AsyncGlobalSecuritiesPages as GlobalSecuritiesPages,
    AsyncIndexAnalyticsPages as IndexAnalyticsPages,
    AsyncMarketSecuritiesPages as MarketSecuritiesPages,
    AsyncMarketTradesPages as MarketTradesPages, AsyncOwnedBoardScope as BoardScope,
    AsyncOwnedEngineScope as EngineScope, AsyncOwnedIndexScope as IndexScope,
    AsyncOwnedMarketScope as MarketScope, AsyncOwnedMarketSecurityScope as MarketSecurityScope,
    AsyncOwnedSecurityResourceScope as SecurityResourceScope,
    AsyncOwnedSecurityScope as SecurityScope, AsyncRawIssRequestBuilder as RawRequest,
    AsyncSecStatsPages as SecStatsPages, AsyncSecuritiesPages as SecuritiesPages,
    AsyncTradesPages as TradesPages,
};
#[cfg(feature = "news")]
/// Пагинаторы новостных endpoint-ов в асинхронном API.
pub use crate::moex::{AsyncEventsPages as EventsPages, AsyncSiteNewsPages as SiteNewsPages};
/// Асинхронный клиент и его builder.
pub use crate::moex::{AsyncMoexClient as Client, AsyncMoexClientBuilder as ClientBuilder};

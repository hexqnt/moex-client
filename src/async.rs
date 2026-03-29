//! Асинхронный API клиента ISS.

#[cfg(feature = "history")]
/// Ленивый paginator по `history` в асинхронном API.
pub use crate::moex::AsyncHistoryPages as HistoryPages;
/// Основные paginator-ы и scope-типы асинхронного API.
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
/// Пагинация новостных endpoint-ов в асинхронном API.
pub use crate::moex::{AsyncEventsPages as EventsPages, AsyncSiteNewsPages as SiteNewsPages};
/// Асинхронный клиент и его builder.
pub use crate::moex::{AsyncMoexClient as Client, AsyncMoexClientBuilder as ClientBuilder};

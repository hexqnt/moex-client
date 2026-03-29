//! Блокирующий API клиента ISS.

#[cfg(feature = "history")]
/// Ленивый пагинатор по `history` в блокирующем API.
pub use crate::moex::HistoryPages;
/// Блокирующий клиент и его builder.
pub use crate::moex::{BlockingMoexClient as Client, BlockingMoexClientBuilder as ClientBuilder};
/// Основные пагинаторы и типы контекстов блокирующего API.
pub use crate::moex::{
    CandlesPages, GlobalSecuritiesPages, IndexAnalyticsPages, MarketSecuritiesPages,
    MarketTradesPages, OwnedBoardScope as BoardScope, OwnedEngineScope as EngineScope,
    OwnedIndexScope as IndexScope, OwnedMarketScope as MarketScope,
    OwnedMarketSecurityScope as MarketSecurityScope,
    OwnedSecurityResourceScope as SecurityResourceScope, OwnedSecurityScope as SecurityScope,
    RawIssRequestBuilder as RawRequest, SecStatsPages, SecuritiesPages, TradesPages,
};
#[cfg(feature = "news")]
/// Пагинаторы новостных endpoint-ов в блокирующем API.
pub use crate::moex::{EventsPages, SiteNewsPages};

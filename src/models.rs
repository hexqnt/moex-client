//! Публичные доменные модели ISS и связанные ошибки парсинга.

mod domain;
mod errors;
mod helpers;
mod identifiers;
mod selectors;
mod wire;

pub use domain::{
    Board, BuySell, Candle, CandleBorder, CandleInterval, CandleOhlcv, CandleQuery, Engine, Event,
    HistoryDates, HistoryQuery, HistoryRecord, Index, IndexAnalytics, Market, OrderbookLevel,
    PageRequest, Pagination, SecStat, Security, SecurityBoard, SecuritySnapshot, SiteNews, Trade,
    Turnover,
};
pub use errors::{
    ParseBoardError, ParseBoardIdError, ParseCandleBorderError, ParseCandleError,
    ParseCandleIntervalError, ParseCandleQueryError, ParseEngineError, ParseEngineNameError,
    ParseEventError, ParseHistoryDatesError, ParseHistoryQueryError, ParseHistoryRecordError,
    ParseIndexAnalyticsError, ParseIndexError, ParseMarketError, ParseMarketNameError,
    ParseOrderbookError, ParseSecIdError, ParseSecStatError, ParseSecurityBoardError,
    ParseSecurityError, ParseSecuritySnapshotError, ParseSiteNewsError, ParseTradeError,
    ParseTurnoverError,
};
pub use identifiers::{BoardId, EngineId, EngineName, IndexId, MarketId, MarketName, SecId};
pub use selectors::{IndexAnalyticsExt, IndexesExt, SecurityBoardsExt, actual_indexes};

pub(crate) use wire::{
    BoardRow, CandleBorderRow, CandleRow, EngineRow, IndexAnalyticsRow, IndexRow, MarketRow,
    OrderbookLevelRow, SecStatRow, SecurityBoardRow, SecurityRow, TradeRow, TurnoverRow,
};
#[cfg(feature = "news")]
pub(crate) use wire::{EventRow, SiteNewsRow};
#[cfg(feature = "history")]
pub(crate) use wire::{HistoryDatesRow, HistoryRow};

#[cfg(test)]
mod tests;

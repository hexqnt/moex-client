//! Публичные доменные модели ISS и связанные ошибки парсинга.

mod domain;
mod selectors;
mod wire;

pub use domain::{
    Board, BoardId, BuySell, Candle, CandleBorder, CandleInterval, CandleOhlcv, CandleQuery,
    Engine, EngineId, EngineName, Event, HistoryDates, HistoryRecord, Index, IndexAnalytics,
    IndexId, Market, MarketId, MarketName, OrderbookLevel, PageRequest, Pagination,
    ParseBoardError, ParseBoardIdError, ParseCandleBorderError, ParseCandleError,
    ParseCandleIntervalError, ParseCandleQueryError, ParseEngineError, ParseEngineNameError,
    ParseEventError, ParseHistoryDatesError, ParseHistoryRecordError, ParseIndexAnalyticsError,
    ParseIndexError, ParseMarketError, ParseMarketNameError, ParseOrderbookError, ParseSecIdError,
    ParseSecStatError, ParseSecurityBoardError, ParseSecurityError, ParseSecuritySnapshotError,
    ParseSiteNewsError, ParseTradeError, ParseTurnoverError, SecId, SecStat, Security,
    SecurityBoard, SecuritySnapshot, SiteNews, Trade, Turnover, actual_indexes,
};
pub use selectors::{IndexAnalyticsExt, IndexesExt, SecurityBoardsExt};

pub(crate) use wire::{
    BoardRow, CandleBorderRow, CandleRow, EngineRow, IndexAnalyticsRow, IndexRow, MarketRow,
    OrderbookLevelRow, SecStatRow, SecurityRow, TradeRow, TurnoverRow,
};
#[cfg(feature = "news")]
pub(crate) use wire::{EventRow, SiteNewsRow};
#[cfg(feature = "history")]
pub(crate) use wire::{HistoryDatesRow, HistoryRow};

#[cfg(test)]
mod tests;

//! Структурированные ошибки построения доменных моделей ISS.

use std::convert::Infallible;

use chrono::{NaiveDate, NaiveDateTime};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Index`](super::Index).
pub enum ParseIndexError {
    /// Пустой `indexid`.
    #[error("index id must not be empty")]
    EmptyIndexId,
    /// Пустое краткое имя индекса.
    #[error("index short name must not be empty")]
    EmptyShortName,
    /// Границы активности индекса заданы в неверном порядке.
    #[error("invalid index date range: from={from} is after till={till}")]
    InvalidDateRange {
        /// Начальная дата периода.
        from: NaiveDate,
        /// Конечная дата периода.
        till: NaiveDate,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`HistoryDates`](super::HistoryDates).
pub enum ParseHistoryDatesError {
    /// Границы доступной истории заданы в неверном порядке.
    #[error("invalid history dates range: from={from} is after till={till}")]
    InvalidDateRange {
        /// Начальная дата доступной истории.
        from: NaiveDate,
        /// Конечная дата доступной истории.
        till: NaiveDate,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`HistoryQuery`](super::HistoryQuery).
pub enum ParseHistoryQueryError {
    /// В запросе истории `from` больше `till`.
    #[error("invalid history query date range: from={from} is after till={till}")]
    InvalidDateRange {
        /// Начальная дата выборки.
        from: NaiveDate,
        /// Конечная дата выборки.
        till: NaiveDate,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`HistoryRecord`](super::HistoryRecord).
pub enum ParseHistoryRecordError {
    /// Некорректный `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Количество сделок отрицательное.
    #[error("history numtrades must not be negative, got {0}")]
    NegativeNumTrades(i64),
    /// Объём торгов отрицательный.
    #[error("history volume must not be negative, got {0}")]
    NegativeVolume(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Turnover`](super::Turnover).
pub enum ParseTurnoverError {
    /// Пустое поле `name`.
    #[error("turnover name must not be empty")]
    EmptyName,
    /// Идентификатор должен быть положительным.
    #[error("turnover id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор не помещается в `u32`.
    #[error("turnover id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// Количество сделок отрицательное.
    #[error("turnover numtrades must not be negative, got {0}")]
    NegativeNumTrades(i64),
    /// Пустое поле `title`.
    #[error("turnover title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`SecStat`](super::SecStat).
pub enum ParseSecStatError {
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Некорректный `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Объём торгов отрицательный.
    #[error("secstats voltoday must not be negative, got {0}")]
    NegativeVolToday(i64),
    /// Количество сделок отрицательное.
    #[error("secstats numtrades must not be negative, got {0}")]
    NegativeNumTrades(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`SiteNews`](super::SiteNews).
pub enum ParseSiteNewsError {
    /// Идентификатор новости должен быть положительным.
    #[error("sitenews id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор новости не помещается в `u64`.
    #[error("sitenews id is out of range for u64, got {0}")]
    IdOutOfRange(i64),
    /// Пустое поле `tag`.
    #[error("sitenews tag must not be empty")]
    EmptyTag,
    /// Пустое поле `title`.
    #[error("sitenews title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Event`](super::Event).
pub enum ParseEventError {
    /// Идентификатор события должен быть положительным.
    #[error("events id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор события не помещается в `u64`.
    #[error("events id is out of range for u64, got {0}")]
    IdOutOfRange(i64),
    /// Пустое поле `tag`.
    #[error("events tag must not be empty")]
    EmptyTag,
    /// Пустое поле `title`.
    #[error("events title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`IndexAnalytics`](super::IndexAnalytics).
pub enum ParseIndexAnalyticsError {
    /// Некорректные данные самого индекса.
    #[error(transparent)]
    InvalidIndexId(#[from] ParseIndexError),
    /// Некорректный `ticker`.
    #[error("ticker is invalid: {0}")]
    InvalidTicker(ParseSecIdError),
    /// Некорректный `secid`.
    #[error("secid is invalid: {0}")]
    InvalidSecId(ParseSecIdError),
    /// Пустое поле `shortnames`.
    #[error("shortnames must not be empty")]
    EmptyShortnames,
    /// Вес компонента не является конечным числом.
    #[error("weight must be finite")]
    NonFiniteWeight,
    /// Вес компонента отрицательный.
    #[error("weight must not be negative")]
    NegativeWeight,
    /// Недопустимое значение `tradingsession`.
    #[error("tradingsession must be 1, 2 or 3, got {0}")]
    InvalidTradingsession(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Engine`](super::Engine).
pub enum ParseEngineError {
    /// Идентификатор движка должен быть положительным.
    #[error("engine id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор движка не помещается в `u32`.
    #[error("engine id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// Некорректное имя движка.
    #[error(transparent)]
    InvalidName(#[from] ParseEngineNameError),
    /// Пустой заголовок движка.
    #[error("engine title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора имени торгового движка.
pub enum ParseEngineNameError {
    /// Пустое имя.
    #[error("engine name must not be empty")]
    Empty,
    /// Имя содержит символ `/`, запрещённый в path-сегменте.
    #[error("engine name must not contain '/'")]
    ContainsSlash,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Market`](super::Market).
pub enum ParseMarketError {
    /// Идентификатор рынка должен быть положительным.
    #[error("market id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор рынка не помещается в `u32`.
    #[error("market id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// Некорректное имя рынка.
    #[error(transparent)]
    InvalidName(#[from] ParseMarketNameError),
    /// Пустой заголовок рынка.
    #[error("market title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Board`](super::Board).
pub enum ParseBoardError {
    /// Идентификатор board должен быть положительным.
    #[error("board id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор board не помещается в `u32`.
    #[error("board id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// `board_group_id` отрицательный.
    #[error("board_group_id must not be negative, got {0}")]
    NegativeBoardGroupId(i64),
    /// `board_group_id` не помещается в `u32`.
    #[error("board_group_id is out of range for u32, got {0}")]
    BoardGroupIdOutOfRange(i64),
    /// Некорректный текстовый `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Пустой заголовок board.
    #[error("board title must not be empty")]
    EmptyTitle,
    /// Некорректный флаг `is_traded` (допустимы только 0/1).
    #[error("is_traded must be 0 or 1, got {0}")]
    InvalidIsTraded(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Security`](super::Security).
pub enum ParseSecurityError {
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Пустое поле `shortname`.
    #[error("security shortname must not be empty")]
    EmptyShortname,
    /// Пустое поле `secname`.
    #[error("security secname must not be empty")]
    EmptySecname,
    /// Пустое поле `status`.
    #[error("security status must not be empty")]
    EmptyStatus,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`SecurityBoard`](super::SecurityBoard).
pub enum ParseSecurityBoardError {
    /// Некорректное имя движка.
    #[error(transparent)]
    InvalidEngine(#[from] ParseEngineNameError),
    /// Некорректное имя рынка.
    #[error(transparent)]
    InvalidMarket(#[from] ParseMarketNameError),
    /// Некорректный `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Некорректный флаг `is_primary` (допустимы только 0/1).
    #[error("is_primary must be 0 or 1, got {0}")]
    InvalidIsPrimary(i64),
}

#[derive(Debug, Error, Clone, PartialEq)]
/// Ошибки построения [`SecuritySnapshot`](super::SecuritySnapshot).
pub enum ParseSecuritySnapshotError {
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Размер лота отрицательный.
    #[error("lot size must not be negative, got {0}")]
    NegativeLotSize(i64),
    /// Размер лота не помещается в `u32`.
    #[error("lot size is out of range for u32, got {0}")]
    LotSizeOutOfRange(i64),
    /// Значение `last` не является конечным числом.
    #[error("last must be finite")]
    NonFiniteLast(f64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Candle`](super::Candle).
pub enum ParseCandleError {
    /// Границы свечи заданы в неверном порядке.
    #[error("invalid candle datetime range: begin={begin} is after end={end}")]
    InvalidDateRange {
        /// Начало свечи.
        begin: NaiveDateTime,
        /// Конец свечи.
        end: NaiveDateTime,
    },
    /// Объём свечи отрицательный.
    #[error("candle volume must not be negative, got {0}")]
    NegativeVolume(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора значения интервала свечей.
pub enum ParseCandleIntervalError {
    /// Неизвестный код интервала ISS.
    #[error("invalid candle interval code, got {0}")]
    InvalidCode(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`CandleQuery`](super::CandleQuery).
pub enum ParseCandleQueryError {
    /// В запросе свечей `from` больше `till`.
    #[error("invalid candle query datetime range: from={from} is after till={till}")]
    InvalidDateRange {
        /// Начальная дата и время выборки.
        from: NaiveDateTime,
        /// Конечная дата и время выборки.
        till: NaiveDateTime,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`CandleBorder`](super::CandleBorder).
pub enum ParseCandleBorderError {
    /// Границы диапазона заданы в неверном порядке.
    #[error("invalid candle borders range: begin={begin} is after end={end}")]
    InvalidDateRange {
        /// Начало доступного диапазона.
        begin: NaiveDateTime,
        /// Конец доступного диапазона.
        end: NaiveDateTime,
    },
    /// Некорректный код интервала.
    #[error(transparent)]
    InvalidInterval(#[from] ParseCandleIntervalError),
    /// `board_group_id` отрицательный.
    #[error("board_group_id must not be negative, got {0}")]
    NegativeBoardGroupId(i64),
    /// `board_group_id` не помещается в `u32`.
    #[error("board_group_id is out of range for u32, got {0}")]
    BoardGroupIdOutOfRange(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Trade`](super::Trade).
pub enum ParseTradeError {
    /// Номер сделки должен быть положительным.
    #[error("trade number must be positive, got {0}")]
    NonPositiveTradeNo(i64),
    /// Номер сделки не помещается в `u64`.
    #[error("trade number is out of range for u64, got {0}")]
    TradeNoOutOfRange(i64),
    /// Количество в сделке отрицательное.
    #[error("trade quantity must not be negative, got {0}")]
    NegativeQuantity(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`OrderbookLevel`](super::OrderbookLevel).
pub enum ParseOrderbookError {
    /// Некорректное направление заявки (`B`/`S`).
    #[error("orderbook side must be 'B' or 'S', got '{0}'")]
    InvalidSide(Box<str>),
    /// Отсутствует цена уровня стакана.
    #[error("orderbook price must be present")]
    MissingPrice,
    /// Цена уровня стакана отрицательная.
    #[error("orderbook price must not be negative")]
    NegativePrice,
    /// Отсутствует объём уровня стакана.
    #[error("orderbook quantity must be present")]
    MissingQuantity,
    /// Объём уровня стакана отрицательный.
    #[error("orderbook quantity must not be negative, got {0}")]
    NegativeQuantity(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора идентификатора инструмента (`secid`).
pub enum ParseSecIdError {
    /// Пустой `secid`.
    #[error("secid must not be empty")]
    Empty,
    /// `secid` содержит символ `/`, запрещённый в path-сегменте.
    #[error("secid must not contain '/'")]
    ContainsSlash,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора идентификатора режима торгов (`boardid`).
pub enum ParseBoardIdError {
    /// Пустой `boardid`.
    #[error("boardid must not be empty")]
    Empty,
    /// `boardid` содержит символ `/`, запрещённый в path-сегменте.
    #[error("boardid must not contain '/'")]
    ContainsSlash,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора имени рынка MOEX.
pub enum ParseMarketNameError {
    /// Пустое имя.
    #[error("market name must not be empty")]
    Empty,
    /// Имя содержит символ `/`, запрещённый в path-сегменте.
    #[error("market name must not contain '/'")]
    ContainsSlash,
}

macro_rules! from_infallible {
    ($($error:ty),+ $(,)?) => {
        $(impl From<Infallible> for $error {
            fn from(value: Infallible) -> Self {
                match value {}
            }
        })+
    };
}

from_infallible!(
    ParseIndexError,
    ParseEngineNameError,
    ParseMarketNameError,
    ParseBoardIdError,
    ParseSecIdError,
);

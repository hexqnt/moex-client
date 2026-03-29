//! Внутренние wire-DTO для таблиц ISS и их преобразование в доменные типы.
//!
//! Здесь намеренно используются tuple-структуры: их порядок полей
//! жёстко синхронизирован с `*.columns` в HTTP-запросах.

use std::borrow::Cow;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Deserialize;

use super::domain::{
    Board, Candle, CandleBorder, CandleOhlcv, Engine, Event, HistoryDates, HistoryRecord,
    HistoryRecordInput, Index, IndexAnalytics, IndexAnalyticsInput, Market, OrderbookLevel,
    ParseBoardError, ParseCandleBorderError, ParseCandleError, ParseEngineError, ParseEventError,
    ParseHistoryDatesError, ParseHistoryRecordError, ParseIndexAnalyticsError, ParseIndexError,
    ParseMarketError, ParseOrderbookError, ParseSecStatError, ParseSecurityError,
    ParseSiteNewsError, ParseTradeError, ParseTurnoverError, SecStat, SecStatInput, Security,
    SiteNews, Trade, Turnover,
};

#[derive(Debug, Deserialize)]
/// Строка таблицы `indices`: `indexid, shortname, from, till`.
pub(crate) struct IndexRow(
    pub(crate) String,
    pub(crate) String,
    #[serde(deserialize_with = "optional_date::deserialize")] pub(crate) Option<NaiveDate>,
    #[serde(deserialize_with = "optional_date::deserialize")] pub(crate) Option<NaiveDate>,
);

impl TryFrom<IndexRow> for Index {
    type Error = ParseIndexError;

    fn try_from(row: IndexRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `history/.../dates`: `from, till`.
pub(crate) struct HistoryDatesRow(
    #[serde(deserialize_with = "date_serde::deserialize")] pub(crate) NaiveDate,
    #[serde(deserialize_with = "date_serde::deserialize")] pub(crate) NaiveDate,
);

impl TryFrom<HistoryDatesRow> for HistoryDates {
    type Error = ParseHistoryDatesError;

    fn try_from(row: HistoryDatesRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `history`.
pub(crate) struct HistoryRow(
    pub(crate) String,
    #[serde(deserialize_with = "date_serde::deserialize")] pub(crate) NaiveDate,
    pub(crate) String,
    pub(crate) Option<i64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
);

impl TryFrom<HistoryRow> for HistoryRecord {
    type Error = ParseHistoryRecordError;

    fn try_from(row: HistoryRow) -> Result<Self, Self::Error> {
        Self::try_new(HistoryRecordInput {
            boardid: row.0,
            tradedate: row.1,
            secid: row.2,
            numtrades: row.3,
            value: row.4,
            open: row.5,
            low: row.6,
            high: row.7,
            close: row.8,
            volume: row.9,
        })
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `turnovers`.
pub(crate) struct TurnoverRow(
    pub(crate) String,
    pub(crate) i64,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    pub(crate) String,
);

impl TryFrom<TurnoverRow> for Turnover {
    type Error = ParseTurnoverError;

    fn try_from(row: TurnoverRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4, row.5, row.6)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `secstats`.
pub(crate) struct SecStatRow(
    pub(crate) String,
    pub(crate) String,
    pub(crate) Option<i64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
    pub(crate) Option<f64>,
);

impl TryFrom<SecStatRow> for SecStat {
    type Error = ParseSecStatError;

    fn try_from(row: SecStatRow) -> Result<Self, Self::Error> {
        Self::try_new(SecStatInput {
            secid: row.0,
            boardid: row.1,
            voltoday: row.2,
            valtoday: row.3,
            highbid: row.4,
            lowoffer: row.5,
            lastoffer: row.6,
            lastbid: row.7,
            open: row.8,
            low: row.9,
            high: row.10,
            last: row.11,
            numtrades: row.12,
            waprice: row.13,
        })
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `sitenews`.
pub(crate) struct SiteNewsRow(
    pub(crate) i64,
    pub(crate) String,
    pub(crate) String,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
);

impl TryFrom<SiteNewsRow> for SiteNews {
    type Error = ParseSiteNewsError;

    fn try_from(row: SiteNewsRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `events`.
pub(crate) struct EventRow(
    pub(crate) i64,
    pub(crate) String,
    pub(crate) String,
    #[serde(deserialize_with = "optional_datetime_serde::deserialize")]
    pub(crate)  Option<NaiveDateTime>,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
);

impl TryFrom<EventRow> for Event {
    type Error = ParseEventError;

    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `analytics`.
pub(crate) struct IndexAnalyticsRow(
    pub(crate) String,
    #[serde(deserialize_with = "date_serde::deserialize")] pub(crate) NaiveDate,
    pub(crate) String,
    pub(crate) String,
    pub(crate) String,
    pub(crate) f64,
    pub(crate) i64,
    #[serde(deserialize_with = "date_serde::deserialize")] pub(crate) NaiveDate,
);

impl TryFrom<IndexAnalyticsRow> for IndexAnalytics {
    type Error = ParseIndexAnalyticsError;

    fn try_from(row: IndexAnalyticsRow) -> Result<Self, Self::Error> {
        Self::try_new(IndexAnalyticsInput {
            indexid: row.0,
            tradedate: row.1,
            ticker: row.2,
            shortnames: row.3,
            secid: row.4,
            weight: row.5,
            tradingsession: row.6,
            trade_session_date: row.7,
        })
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `engines`: `id, name, title`.
pub(crate) struct EngineRow(pub(crate) i64, pub(crate) String, pub(crate) String);

impl TryFrom<EngineRow> for Engine {
    type Error = ParseEngineError;

    fn try_from(row: EngineRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `markets`: `id, NAME, title`.
pub(crate) struct MarketRow(pub(crate) i64, pub(crate) String, pub(crate) String);

impl TryFrom<MarketRow> for Market {
    type Error = ParseMarketError;

    fn try_from(row: MarketRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `boards`.
pub(crate) struct BoardRow(
    pub(crate) i64,
    pub(crate) i64,
    pub(crate) String,
    pub(crate) String,
    pub(crate) i64,
);

impl TryFrom<BoardRow> for Board {
    type Error = ParseBoardError;

    fn try_from(row: BoardRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `securities`.
pub(crate) struct SecurityRow(
    pub(crate) String,
    pub(crate) String,
    pub(crate) String,
    pub(crate) String,
);

impl TryFrom<SecurityRow> for Security {
    type Error = ParseSecurityError;

    fn try_from(row: SecurityRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `candleborders`.
pub(crate) struct CandleBorderRow(
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    pub(crate) i64,
    pub(crate) i64,
);

impl TryFrom<CandleBorderRow> for CandleBorder {
    type Error = ParseCandleBorderError;

    fn try_from(row: CandleBorderRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `candles`.
pub(crate) struct CandleRow(
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
);

impl TryFrom<CandleRow> for Candle {
    type Error = ParseCandleError;

    fn try_from(row: CandleRow) -> Result<Self, Self::Error> {
        Self::try_new(
            row.0,
            row.1,
            CandleOhlcv::new(row.2, row.3, row.4, row.5, row.6, row.7),
        )
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `trades`.
pub(crate) struct TradeRow(
    pub(crate) i64,
    #[serde(deserialize_with = "time_serde::deserialize")] pub(crate) NaiveTime,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
    pub(crate) Option<f64>,
);

impl TryFrom<TradeRow> for Trade {
    type Error = ParseTradeError;

    fn try_from(row: TradeRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

#[derive(Debug, Deserialize)]
/// Строка таблицы `orderbook`.
pub(crate) struct OrderbookLevelRow(
    pub(crate) String,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
);

impl TryFrom<OrderbookLevelRow> for OrderbookLevel {
    type Error = ParseOrderbookError;

    fn try_from(row: OrderbookLevelRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2)
    }
}

mod optional_date {
    use super::*;
    use serde::{Deserialize, Deserializer};

    /// Десериализация optional-даты ISS:
    /// `null` и пустая строка трактуются как отсутствие значения.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Option::<Cow<'de, str>>::deserialize(deserializer)?;
        match raw {
            None => Ok(None),
            Some(raw) => {
                let raw = raw.trim();
                if raw.is_empty() {
                    Ok(None)
                } else {
                    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
                        .map(Some)
                        .map_err(serde::de::Error::custom)
                }
            }
        }
    }
}

mod datetime_serde {
    use super::*;
    use serde::{Deserialize, Deserializer};

    const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    /// Десериализация даты и времени в формате ISS `%Y-%m-%d %H:%M:%S`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Cow::<str>::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(raw.trim(), DATETIME_FORMAT).map_err(serde::de::Error::custom)
    }
}

mod optional_datetime_serde {
    use super::*;
    use serde::{Deserialize, Deserializer};

    const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    /// Десериализация optional-datetime ISS:
    /// `null` и пустая строка трактуются как отсутствие значения.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Option::<Cow<'de, str>>::deserialize(deserializer)?;
        match raw {
            None => Ok(None),
            Some(raw) => {
                let raw = raw.trim();
                if raw.is_empty() {
                    Ok(None)
                } else {
                    NaiveDateTime::parse_from_str(raw, DATETIME_FORMAT)
                        .map(Some)
                        .map_err(serde::de::Error::custom)
                }
            }
        }
    }
}

mod date_serde {
    use super::*;
    use serde::{Deserialize, Deserializer};

    const DATE_FORMAT: &str = "%Y-%m-%d";

    /// Десериализация даты ISS в формате `%Y-%m-%d`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Cow::<str>::deserialize(deserializer)?;
        NaiveDate::parse_from_str(raw.trim(), DATE_FORMAT).map_err(serde::de::Error::custom)
    }
}

mod time_serde {
    use super::*;
    use serde::{Deserialize, Deserializer};

    const TIME_FORMAT: &str = "%H:%M:%S";

    /// Десериализация времени ISS в формате `%H:%M:%S`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Cow::<str>::deserialize(deserializer)?;
        NaiveTime::parse_from_str(raw.trim(), TIME_FORMAT).map_err(serde::de::Error::custom)
    }
}

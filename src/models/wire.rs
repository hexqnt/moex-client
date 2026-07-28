//! Внутренние wire DTO для таблиц ISS и их преобразование в доменные типы.
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

mod optional_date {
    use serde::{Deserialize, Deserializer};

    use super::*;

    /// Десериализация опциональной даты ISS.
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
    use serde::{Deserialize, Deserializer};

    use super::*;

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
    use serde::{Deserialize, Deserializer};

    use super::*;

    const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    /// Десериализация опционального ISS datetime.
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
    use serde::{Deserialize, Deserializer};

    use super::*;

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
    use serde::{Deserialize, Deserializer};

    use super::*;

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
#[derive(Debug, Deserialize)]
/// Строка таблицы `indices`: `indexid, shortname, from, till`.
pub(crate) struct IndexRow(
    pub(crate) String,
    pub(crate) String,
    #[serde(deserialize_with = "optional_date::deserialize")] pub(crate) Option<NaiveDate>,
    #[serde(deserialize_with = "optional_date::deserialize")] pub(crate) Option<NaiveDate>,
);

#[derive(Debug, Deserialize)]
/// Строка таблицы `history/.../dates`: `from, till`.
pub(crate) struct HistoryDatesRow(
    #[serde(deserialize_with = "date_serde::deserialize")] pub(crate) NaiveDate,
    #[serde(deserialize_with = "date_serde::deserialize")] pub(crate) NaiveDate,
);

#[derive(Debug, Deserialize)]
/// Строка таблицы `history`.
pub(crate) struct HistoryRow {
    #[serde(rename = "BOARDID")]
    pub(crate) boardid: String,
    #[serde(rename = "TRADEDATE", deserialize_with = "date_serde::deserialize")]
    pub(crate) tradedate: NaiveDate,
    #[serde(rename = "SECID")]
    pub(crate) secid: String,
    #[serde(rename = "NUMTRADES", default)]
    pub(crate) numtrades: Option<i64>,
    #[serde(rename = "VALUE", default)]
    pub(crate) value: Option<f64>,
    #[serde(rename = "OPEN", default)]
    pub(crate) open: Option<f64>,
    #[serde(rename = "LOW", default)]
    pub(crate) low: Option<f64>,
    #[serde(rename = "HIGH", default)]
    pub(crate) high: Option<f64>,
    #[serde(rename = "CLOSE", default)]
    pub(crate) close: Option<f64>,
    #[serde(rename = "VOLUME", default)]
    pub(crate) volume: Option<i64>,
    #[serde(rename = "DURATION", default)]
    pub(crate) duration_days: Option<f64>,
    #[serde(rename = "YIELD", default)]
    pub(crate) yield_percent: Option<f64>,
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

#[derive(Debug, Deserialize)]
/// Строка таблицы `sitenews`.
pub(crate) struct SiteNewsRow(
    pub(crate) i64,
    pub(crate) String,
    pub(crate) String,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
);

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

#[derive(Debug, Deserialize)]
/// Строка таблицы `engines`: `id, name, title`.
pub(crate) struct EngineRow(pub(crate) i64, pub(crate) String, pub(crate) String);

#[derive(Debug, Deserialize)]
/// Строка таблицы `markets`: `id, NAME, title`.
pub(crate) struct MarketRow(pub(crate) i64, pub(crate) String, pub(crate) String);

#[derive(Debug, Deserialize)]
/// Строка таблицы `boards`.
pub(crate) struct BoardRow(
    pub(crate) i64,
    pub(crate) i64,
    pub(crate) String,
    pub(crate) String,
    pub(crate) i64,
);

#[derive(Debug, Deserialize)]
/// Строка таблицы `securities`.
pub(crate) struct SecurityRow(
    pub(crate) String,
    pub(crate) String,
    pub(crate) String,
    pub(crate) String,
);

#[derive(Debug, Deserialize)]
/// Строка таблицы `candleborders`.
pub(crate) struct CandleBorderRow(
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    #[serde(deserialize_with = "datetime_serde::deserialize")] pub(crate) NaiveDateTime,
    pub(crate) i64,
    pub(crate) i64,
);

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

#[derive(Debug, Deserialize)]
/// Строка таблицы `trades`.
pub(crate) struct TradeRow(
    pub(crate) i64,
    #[serde(deserialize_with = "time_serde::deserialize")] pub(crate) NaiveTime,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
    pub(crate) Option<f64>,
);

#[derive(Debug, Deserialize)]
/// Строка таблицы `orderbook`.
pub(crate) struct OrderbookLevelRow(
    pub(crate) String,
    pub(crate) Option<f64>,
    pub(crate) Option<i64>,
);

impl TryFrom<IndexRow> for Index {
    type Error = ParseIndexError;

    fn try_from(row: IndexRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3)
    }
}

impl TryFrom<HistoryDatesRow> for HistoryDates {
    type Error = ParseHistoryDatesError;

    fn try_from(row: HistoryDatesRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1)
    }
}

impl TryFrom<HistoryRow> for HistoryRecord {
    type Error = ParseHistoryRecordError;

    fn try_from(row: HistoryRow) -> Result<Self, Self::Error> {
        Self::try_new(HistoryRecordInput {
            boardid: row.boardid,
            tradedate: row.tradedate,
            secid: row.secid,
            numtrades: row.numtrades,
            value: row.value,
            open: row.open,
            low: row.low,
            high: row.high,
            close: row.close,
            volume: row.volume,
            duration_days: row.duration_days,
            yield_percent: row.yield_percent,
        })
    }
}

impl TryFrom<TurnoverRow> for Turnover {
    type Error = ParseTurnoverError;

    fn try_from(row: TurnoverRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4, row.5, row.6)
    }
}

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

impl TryFrom<SiteNewsRow> for SiteNews {
    type Error = ParseSiteNewsError;

    fn try_from(row: SiteNewsRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

impl TryFrom<EventRow> for Event {
    type Error = ParseEventError;

    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

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

impl TryFrom<EngineRow> for Engine {
    type Error = ParseEngineError;

    fn try_from(row: EngineRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2)
    }
}

impl TryFrom<MarketRow> for Market {
    type Error = ParseMarketError;

    fn try_from(row: MarketRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2)
    }
}

impl TryFrom<BoardRow> for Board {
    type Error = ParseBoardError;

    fn try_from(row: BoardRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

impl TryFrom<SecurityRow> for Security {
    type Error = ParseSecurityError;

    fn try_from(row: SecurityRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3)
    }
}

impl TryFrom<CandleBorderRow> for CandleBorder {
    type Error = ParseCandleBorderError;

    fn try_from(row: CandleBorderRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3)
    }
}

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

impl TryFrom<TradeRow> for Trade {
    type Error = ParseTradeError;

    fn try_from(row: TradeRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2, row.3, row.4)
    }
}

impl TryFrom<OrderbookLevelRow> for OrderbookLevel {
    type Error = ParseOrderbookError;

    fn try_from(row: OrderbookLevelRow) -> Result<Self, Self::Error> {
        Self::try_new(row.0, row.1, row.2)
    }
}

//! Внутренние wire DTO для таблиц ISS.
//!
//! Здесь намеренно используются tuple-структуры: их порядок полей
//! жёстко синхронизирован с `*.columns` в HTTP-запросах.

use std::borrow::Cow;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Deserializer};

// Заимствуем неэкранированный JSON-текст; для escape-последовательностей и owned-ввода serde сохраняет владение.
#[derive(Deserialize)]
#[serde(transparent)]
struct IssText<'a>(#[serde(borrow)] Cow<'a, str>);

macro_rules! temporal_serde {
    ($module:ident, $value:ty, $format:literal) => {
        mod $module {
            use super::*;

            pub fn deserialize<'de, D>(deserializer: D) -> Result<$value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = IssText::deserialize(deserializer)?.0;
                <$value>::parse_from_str(raw.trim(), $format).map_err(serde::de::Error::custom)
            }
        }
    };
    (optional $module:ident, $value:ty, $format:literal) => {
        mod $module {
            use super::*;

            pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<$value>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = Option::<IssText<'de>>::deserialize(deserializer)?;
                match raw {
                    None => Ok(None),
                    Some(raw) => {
                        let raw = raw.0.trim();
                        if raw.is_empty() {
                            Ok(None)
                        } else {
                            <$value>::parse_from_str(raw, $format)
                                .map(Some)
                                .map_err(serde::de::Error::custom)
                        }
                    }
                }
            }
        }
    };
}

temporal_serde!(optional optional_date, NaiveDate, "%Y-%m-%d");
temporal_serde!(datetime_serde, NaiveDateTime, "%Y-%m-%d %H:%M:%S");
temporal_serde!(optional optional_datetime_serde, NaiveDateTime, "%Y-%m-%d %H:%M:%S");
temporal_serde!(date_serde, NaiveDate, "%Y-%m-%d");
temporal_serde!(time_serde, NaiveTime, "%H:%M:%S");

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

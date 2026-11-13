//! Внутренние wire DTO для таблиц ISS.
//!
//! Здесь намеренно используются tuple-структуры: их порядок полей
//! жёстко синхронизирован с `*.columns` в HTTP-запросах.

use std::borrow::Cow;

use chrono::format::{Item, Numeric, Pad, Parsed, parse};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, ParseResult};
use serde::{Deserialize, Deserializer};

// Статический datetime-формат и его date/time-срезы исключают повторный разбор strftime для каждого поля.
const DATETIME_ITEMS: &[Item<'static>] = &[
    Item::Numeric(Numeric::Year, Pad::Zero),
    Item::Literal("-"),
    Item::Numeric(Numeric::Month, Pad::Zero),
    Item::Literal("-"),
    Item::Numeric(Numeric::Day, Pad::Zero),
    Item::Space(" "),
    Item::Numeric(Numeric::Hour, Pad::Zero),
    Item::Literal(":"),
    Item::Numeric(Numeric::Minute, Pad::Zero),
    Item::Literal(":"),
    Item::Numeric(Numeric::Second, Pad::Zero),
];
const DATE_ITEMS: &[Item<'static>] = DATETIME_ITEMS.split_at(5).0;
const TIME_ITEMS: &[Item<'static>] = DATETIME_ITEMS.split_at(6).1;

// Заимствуем неэкранированный JSON-текст; для escape-последовательностей и owned-ввода serde сохраняет владение.
#[derive(Deserialize)]
#[serde(transparent)]
struct IssText<'a>(#[serde(borrow)] Cow<'a, str>);

fn parse_temporal<T>(
    raw: &str,
    items: &[Item<'_>],
    convert: impl FnOnce(&Parsed) -> ParseResult<T>,
) -> ParseResult<T> {
    let mut parsed = Parsed::new();
    parse(&mut parsed, raw, items.iter())?;
    convert(&parsed)
}

macro_rules! temporal_serde {
    ($module:ident, $value:ty, $items:ident, $convert:expr) => {
        mod $module {
            use super::*;

            pub fn deserialize<'de, D>(deserializer: D) -> Result<$value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = IssText::deserialize(deserializer)?.0;
                parse_temporal(raw.trim(), $items, $convert).map_err(serde::de::Error::custom)
            }
        }
    };
    (optional $module:ident, $value:ty, $items:ident, $convert:expr) => {
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
                            parse_temporal(raw, $items, $convert)
                                .map(Some)
                                .map_err(serde::de::Error::custom)
                        }
                    }
                }
            }
        }
    };
}

temporal_serde!(optional optional_date, NaiveDate, DATE_ITEMS, Parsed::to_naive_date);
temporal_serde!(datetime_serde, NaiveDateTime, DATETIME_ITEMS, |parsed| {
    parsed.to_naive_datetime_with_offset(0)
});
temporal_serde!(optional optional_datetime_serde, NaiveDateTime, DATETIME_ITEMS, |parsed| parsed.to_naive_datetime_with_offset(0));
temporal_serde!(date_serde, NaiveDate, DATE_ITEMS, Parsed::to_naive_date);
temporal_serde!(time_serde, NaiveTime, TIME_ITEMS, Parsed::to_naive_time);

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
/// Строка таблицы `boards` внутри `securities/{secid}`.
pub(crate) struct SecurityBoardRow(
    pub(crate) String,
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

#[cfg(test)]
mod tests;

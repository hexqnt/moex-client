use crate::models::{
    Board, Candle, CandleBorder, Engine, Index, IndexAnalytics, Market, OrderbookLevel, SecStat,
    Security, SecurityBoard, SecuritySnapshot, Trade, Turnover,
};
#[cfg(feature = "news")]
use crate::models::{Event, SiteNews};
#[cfg(feature = "history")]
use crate::models::{HistoryDates, HistoryRecord};

use super::MoexError;
use super::constants::{
    BOARDS_ENDPOINT_TEMPLATE, CANDLEBORDERS_ENDPOINT_TEMPLATE, CANDLES_ENDPOINT_TEMPLATE,
    INDEX_ANALYTICS_ENDPOINT_TEMPLATE, MARKETS_ENDPOINT_TEMPLATE, ORDERBOOK_ENDPOINT_TEMPLATE,
    SECSTATS_ENDPOINT_TEMPLATE, SECURITIES_ENDPOINT_TEMPLATE, SECURITY_BOARDS_ENDPOINT_TEMPLATE,
    TRADES_ENDPOINT_TEMPLATE, TURNOVERS_ENDPOINT,
};
#[cfg(feature = "news")]
use super::constants::{EVENTS_ENDPOINT, SITENEWS_ENDPOINT};
#[cfg(feature = "history")]
use super::constants::{HISTORY_DATES_ENDPOINT_TEMPLATE, HISTORY_ENDPOINT_TEMPLATE};
pub use super::payload::{RawTableView, RawTables};
use super::payload::{
    decode_board_security_snapshots_json_with_endpoint, decode_boards_json_with_endpoint,
    decode_candle_borders_json_with_endpoint, decode_candles_json_with_endpoint,
    decode_engines_json_payload, decode_index_analytics_json_with_endpoint,
    decode_indexes_json_payload, decode_markets_json_with_endpoint,
    decode_orderbook_json_with_endpoint, decode_raw_table_rows_json_with_endpoint,
    decode_raw_table_view_json_with_endpoint, decode_raw_tables_json_with_endpoint,
    decode_secstats_json_with_endpoint, decode_securities_json_with_endpoint,
    decode_security_boards_json_with_endpoint, decode_trades_json_with_endpoint,
    decode_turnovers_json_with_endpoint,
};
#[cfg(feature = "news")]
use super::payload::{decode_events_json_with_endpoint, decode_sitenews_json_with_endpoint};
#[cfg(feature = "history")]
use super::payload::{decode_history_dates_json_with_endpoint, decode_history_json_with_endpoint};

/// Разобрать JSON-представление `indices` ISS в доменные типы.
pub fn indexes_json(payload: &str) -> Result<Vec<Index>, MoexError> {
    decode_indexes_json_payload(payload)
}

/// Разобрать JSON-представление `engines` ISS в доменные типы.
pub fn engines_json(payload: &str) -> Result<Vec<Engine>, MoexError> {
    decode_engines_json_payload(payload)
}

/// Разобрать JSON-представление `markets` ISS в доменные типы.
pub fn markets_json(payload: &str) -> Result<Vec<Market>, MoexError> {
    decode_markets_json_with_endpoint(payload, MARKETS_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `boards` ISS в доменные типы.
pub fn boards_json(payload: &str) -> Result<Vec<Board>, MoexError> {
    decode_boards_json_with_endpoint(payload, BOARDS_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON `boards` из `securities/{secid}` в доменные типы.
pub fn security_boards_json(payload: &str) -> Result<Vec<SecurityBoard>, MoexError> {
    decode_security_boards_json_with_endpoint(payload, SECURITY_BOARDS_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `securities` ISS в доменные типы.
pub fn securities_json(payload: &str) -> Result<Vec<Security>, MoexError> {
    decode_securities_json_with_endpoint(payload, SECURITIES_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON `securities+marketdata` в снимки инструментов.
pub fn board_security_snapshots_json(payload: &str) -> Result<Vec<SecuritySnapshot>, MoexError> {
    decode_board_security_snapshots_json_with_endpoint(payload, SECURITIES_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `orderbook` ISS в доменные типы.
pub fn orderbook_json(payload: &str) -> Result<Vec<OrderbookLevel>, MoexError> {
    decode_orderbook_json_with_endpoint(payload, ORDERBOOK_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `candleborders` ISS в доменные типы.
pub fn candle_borders_json(payload: &str) -> Result<Vec<CandleBorder>, MoexError> {
    decode_candle_borders_json_with_endpoint(payload, CANDLEBORDERS_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `candles` ISS в доменные типы.
pub fn candles_json(payload: &str) -> Result<Vec<Candle>, MoexError> {
    decode_candles_json_with_endpoint(payload, CANDLES_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `trades` ISS в доменные типы.
pub fn trades_json(payload: &str) -> Result<Vec<Trade>, MoexError> {
    decode_trades_json_with_endpoint(payload, TRADES_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `analytics` ISS в доменные типы.
pub fn index_analytics_json(payload: &str) -> Result<Vec<IndexAnalytics>, MoexError> {
    decode_index_analytics_json_with_endpoint(payload, INDEX_ANALYTICS_ENDPOINT_TEMPLATE)
}

/// Разобрать JSON-представление `turnovers` ISS в доменные типы.
pub fn turnovers_json(payload: &str) -> Result<Vec<Turnover>, MoexError> {
    decode_turnovers_json_with_endpoint(payload, TURNOVERS_ENDPOINT)
}

/// Разобрать JSON-представление `secstats` ISS в доменные типы.
pub fn secstats_json(payload: &str) -> Result<Vec<SecStat>, MoexError> {
    decode_secstats_json_with_endpoint(payload, SECSTATS_ENDPOINT_TEMPLATE)
}

#[cfg(feature = "history")]
/// Разобрать JSON-представление `history/.../dates` ISS в доменные типы.
pub fn history_dates_json(payload: &str) -> Result<Vec<HistoryDates>, MoexError> {
    decode_history_dates_json_with_endpoint(payload, HISTORY_DATES_ENDPOINT_TEMPLATE)
}

#[cfg(feature = "history")]
/// Разобрать JSON-представление `history` ISS в доменные типы.
pub fn history_json(payload: &str) -> Result<Vec<HistoryRecord>, MoexError> {
    decode_history_json_with_endpoint(payload, HISTORY_ENDPOINT_TEMPLATE)
}

#[cfg(feature = "news")]
/// Разобрать JSON-представление `sitenews` ISS в доменные типы.
pub fn sitenews_json(payload: &str) -> Result<Vec<SiteNews>, MoexError> {
    decode_sitenews_json_with_endpoint(payload, SITENEWS_ENDPOINT)
}

#[cfg(feature = "news")]
/// Разобрать JSON-представление `events` ISS в доменные типы.
pub fn events_json(payload: &str) -> Result<Vec<Event>, MoexError> {
    decode_events_json_with_endpoint(payload, EVENTS_ENDPOINT)
}

/// Декодировать строки выбранной ISS-таблицы в пользовательский тип.
///
/// Аргумент `endpoint` используется для контекста ошибок.
/// Если нужен только один блок, это самый прямой API.
pub fn raw_table_rows_json<T>(
    payload: &str,
    endpoint: &str,
    table: &str,
) -> Result<Vec<T>, MoexError>
where
    T: serde::de::DeserializeOwned,
{
    decode_raw_table_rows_json_with_endpoint(payload, endpoint, table)
}

/// Декодировать таблицу ISS в borrowed-представление без `DeserializeOwned`.
///
/// Подходит для zero-copy чтения отдельных ячеек и ленивой десериализации.
pub fn raw_table_view_json<'a>(
    payload: &'a str,
    endpoint: &str,
    table: &str,
) -> Result<RawTableView<'a>, MoexError> {
    decode_raw_table_view_json_with_endpoint(payload, endpoint, table)
}

/// Подготовить верхнеуровневые блоки payload-а для декодирования нескольких таблиц.
///
/// Полезно, когда из одного payload-а нужно извлечь несколько таблиц
/// без повторного разбора всего JSON.
/// Дальше используйте [`RawTables::take_rows`] для поэтапного извлечения.
pub fn raw_tables_json(payload: &str, endpoint: &str) -> Result<RawTables, MoexError> {
    decode_raw_tables_json_with_endpoint(payload, endpoint)
}

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
    ENGINES_ENDPOINT, INDEX_ANALYTICS_ENDPOINT_TEMPLATE, INDEXES_ENDPOINT,
    MARKETS_ENDPOINT_TEMPLATE, ORDERBOOK_ENDPOINT_TEMPLATE, SECSTATS_ENDPOINT_TEMPLATE,
    SECURITIES_ENDPOINT_TEMPLATE, SECURITY_BOARDS_ENDPOINT_TEMPLATE, TRADES_ENDPOINT_TEMPLATE,
    TURNOVERS_ENDPOINT,
};
#[cfg(feature = "news")]
use super::constants::{EVENTS_ENDPOINT, SITENEWS_ENDPOINT};
#[cfg(feature = "history")]
use super::constants::{HISTORY_DATES_ENDPOINT_TEMPLATE, HISTORY_ENDPOINT_TEMPLATE};
use super::payload;

pub use super::payload::{RawTableView, RawTables};

macro_rules! decoders {
    ($($(#[$attr:meta])* $name:ident: $model:ty => $decode:ident, $endpoint:ident;)*) => {
        $(
            $(#[$attr])*
            pub fn $name(payload: &str) -> Result<Vec<$model>, MoexError> {
                payload::$decode(payload, $endpoint)
            }
        )*
    };
}

decoders! {
    /// Разобрать JSON-представление `indices` ISS в доменные типы.
    indexes_json: Index => decode_indexes_json_with_endpoint, INDEXES_ENDPOINT;
    /// Разобрать JSON-представление `engines` ISS в доменные типы.
    engines_json: Engine => decode_engines_json_with_endpoint, ENGINES_ENDPOINT;
    /// Разобрать JSON-представление `markets` ISS в доменные типы.
    markets_json: Market => decode_markets_json_with_endpoint, MARKETS_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `boards` ISS в доменные типы.
    boards_json: Board => decode_boards_json_with_endpoint, BOARDS_ENDPOINT_TEMPLATE;
    /// Разобрать JSON `boards` из `securities/{secid}` в доменные типы.
    security_boards_json: SecurityBoard => decode_security_boards_json_with_endpoint, SECURITY_BOARDS_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `securities` ISS в доменные типы.
    securities_json: Security => decode_securities_json_with_endpoint, SECURITIES_ENDPOINT_TEMPLATE;
    /// Разобрать JSON `securities+marketdata` в снимки инструментов.
    board_security_snapshots_json: SecuritySnapshot => decode_board_security_snapshots_json_with_endpoint, SECURITIES_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `orderbook` ISS в доменные типы.
    orderbook_json: OrderbookLevel => decode_orderbook_json_with_endpoint, ORDERBOOK_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `candleborders` ISS в доменные типы.
    candle_borders_json: CandleBorder => decode_candle_borders_json_with_endpoint, CANDLEBORDERS_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `candles` ISS в доменные типы.
    candles_json: Candle => decode_candles_json_with_endpoint, CANDLES_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `trades` ISS в доменные типы.
    trades_json: Trade => decode_trades_json_with_endpoint, TRADES_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `analytics` ISS в доменные типы.
    index_analytics_json: IndexAnalytics => decode_index_analytics_json_with_endpoint, INDEX_ANALYTICS_ENDPOINT_TEMPLATE;
    /// Разобрать JSON-представление `turnovers` ISS в доменные типы.
    turnovers_json: Turnover => decode_turnovers_json_with_endpoint, TURNOVERS_ENDPOINT;
    /// Разобрать JSON-представление `secstats` ISS в доменные типы.
    secstats_json: SecStat => decode_secstats_json_with_endpoint, SECSTATS_ENDPOINT_TEMPLATE;
    #[cfg(feature = "history")]
    /// Разобрать JSON-представление `history/.../dates` ISS в доменные типы.
    history_dates_json: HistoryDates => decode_history_dates_json_with_endpoint, HISTORY_DATES_ENDPOINT_TEMPLATE;
    #[cfg(feature = "history")]
    /// Разобрать JSON-представление `history` ISS в доменные типы.
    history_json: HistoryRecord => decode_history_json_with_endpoint, HISTORY_ENDPOINT_TEMPLATE;
    #[cfg(feature = "news")]
    /// Разобрать JSON-представление `sitenews` ISS в доменные типы.
    sitenews_json: SiteNews => decode_sitenews_json_with_endpoint, SITENEWS_ENDPOINT;
    #[cfg(feature = "news")]
    /// Разобрать JSON-представление `events` ISS в доменные типы.
    events_json: Event => decode_events_json_with_endpoint, EVENTS_ENDPOINT;
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
    payload::decode_raw_table_rows_json_with_endpoint(payload, endpoint, table)
}

/// Декодировать таблицу ISS в borrowed-представление без `DeserializeOwned`.
///
/// Подходит для zero-copy чтения отдельных ячеек и ленивой десериализации.
pub fn raw_table_view_json<'a>(
    payload: &'a str,
    endpoint: &str,
    table: &str,
) -> Result<RawTableView<'a>, MoexError> {
    payload::decode_raw_table_view_json_with_endpoint(payload, endpoint, table)
}

/// Подготовить верхнеуровневые блоки payload-а для декодирования нескольких таблиц.
///
/// Полезно, когда из одного payload-а нужно извлечь несколько таблиц
/// без повторного разбора всего JSON.
/// Дальше используйте [`RawTables::take_rows`] для поэтапного извлечения.
pub fn raw_tables_json(payload: &str, endpoint: &str) -> Result<RawTables, MoexError> {
    payload::decode_raw_tables_json_with_endpoint(payload, endpoint)
}

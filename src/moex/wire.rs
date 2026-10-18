//! Wire-структуры верхнего уровня для JSON-ответов ISS.
//!
//! Эти типы точно повторяют форму JSON, которую возвращает API:
//! таблицы вида `{ "columns": [...], "data": [...] }` инкапсулированы в [`IssTable`].

use serde::Deserialize;

#[cfg(feature = "history")]
use crate::models::HistoryDatesRow;
use crate::models::{
    BoardRow, CandleBorderRow, CandleRow, EngineRow, IndexAnalyticsRow, IndexRow, MarketRow,
    OrderbookLevelRow, SecStatRow, SecurityRow, TradeRow, TurnoverRow,
};
#[cfg(feature = "news")]
use crate::models::{EventRow, SiteNewsRow};

#[derive(Debug, Deserialize)]
/// Универсальное представление табличного блока ISS (`...: { data: [...] }`).
pub(super) struct IssTable<T> {
    pub(super) data: Vec<T>,
}

// Сохраняем отдельные derive-типы: serde диагностирует отсутствующие и повторные блоки.
macro_rules! table_responses {
    ($($(#[$attr:meta])* $name:ident: $table:ident => $row:ty;)*) => {
        $(
            $(#[$attr])*
            #[derive(Debug, Deserialize)]
            pub(super) struct $name {
                pub(super) $table: IssTable<$row>,
            }
        )*
    };
}

table_responses! {
    IndexesResponse: indices => IndexRow;
    #[cfg(feature = "history")]
    HistoryDatesResponse: dates => HistoryDatesRow;
    TurnoversResponse: turnovers => TurnoverRow;
    #[cfg(feature = "news")]
    SiteNewsResponse: sitenews => SiteNewsRow;
    #[cfg(feature = "news")]
    EventsResponse: events => EventRow;
    SecStatsResponse: secstats => SecStatRow;
    IndexAnalyticsResponse: analytics => IndexAnalyticsRow;
    EnginesResponse: engines => EngineRow;
    MarketsResponse: markets => MarketRow;
    BoardsResponse: boards => BoardRow;
    SecuritiesResponse: securities => SecurityRow;
    OrderbookResponse: orderbook => OrderbookLevelRow;
    CandleBordersResponse: borders => CandleBorderRow;
    CandlesResponse: candles => CandleRow;
    TradesResponse: trades => TradeRow;
    SecurityBoardsResponse: boards => SecurityBoardRow;
}

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `boards` внутри `securities/{secid}`.
pub(super) struct SecurityBoardRow(
    pub(super) String,
    pub(super) String,
    pub(super) String,
    pub(super) i64,
);

#[derive(Debug, Deserialize)]
/// Ответ объединённого endpoint `securities + marketdata`.
pub(super) struct BoardSecuritySnapshotsResponse {
    pub(super) securities: IssTable<BoardSecurityRow>,
    pub(super) marketdata: IssTable<BoardMarketDataRow>,
}

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `securities` для режима snapshot.
pub(super) struct BoardSecurityRow(pub(super) String, pub(super) Option<i64>);

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `marketdata` для режима snapshot.
pub(super) struct BoardMarketDataRow(pub(super) String, pub(super) Option<f64>);

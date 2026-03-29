//! Wire-структуры верхнего уровня для JSON-ответов ISS.
//!
//! Эти типы отражают ровно ту форму JSON, которую возвращает API:
//! таблицы вида `{ "columns": [...], "data": [...] }` инкапсулированы в [`IssTable`].

use serde::Deserialize;

use crate::models::{
    BoardRow, CandleBorderRow, CandleRow, EngineRow, IndexAnalyticsRow, IndexRow, MarketRow,
    OrderbookLevelRow, SecStatRow, SecurityRow, TradeRow, TurnoverRow,
};
#[cfg(feature = "news")]
use crate::models::{EventRow, SiteNewsRow};
#[cfg(feature = "history")]
use crate::models::{HistoryDatesRow, HistoryRow};

#[derive(Debug, Deserialize)]
/// Универсальное представление табличного блока ISS (`...: { data: [...] }`).
pub(super) struct IssTable<T> {
    pub(super) data: Vec<T>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `indices`.
pub(super) struct IndexesResponse {
    pub(super) indices: IssTable<IndexRow>,
}

#[cfg(feature = "history")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `history/.../dates`.
pub(super) struct HistoryDatesResponse {
    pub(super) dates: IssTable<HistoryDatesRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `turnovers`.
pub(super) struct TurnoversResponse {
    pub(super) turnovers: IssTable<TurnoverRow>,
}

#[cfg(feature = "news")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `sitenews`.
pub(super) struct SiteNewsResponse {
    pub(super) sitenews: IssTable<SiteNewsRow>,
}

#[cfg(feature = "news")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `events`.
pub(super) struct EventsResponse {
    pub(super) events: IssTable<EventRow>,
}

#[cfg(feature = "history")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `history`.
pub(super) struct HistoryResponse {
    pub(super) history: IssTable<HistoryRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `secstats`.
pub(super) struct SecStatsResponse {
    pub(super) secstats: IssTable<SecStatRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `index_analytics`.
pub(super) struct IndexAnalyticsResponse {
    pub(super) analytics: IssTable<IndexAnalyticsRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `engines`.
pub(super) struct EnginesResponse {
    pub(super) engines: IssTable<EngineRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `markets`.
pub(super) struct MarketsResponse {
    pub(super) markets: IssTable<MarketRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `boards`.
pub(super) struct BoardsResponse {
    pub(super) boards: IssTable<BoardRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `securities`.
pub(super) struct SecuritiesResponse {
    pub(super) securities: IssTable<SecurityRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `orderbook`.
pub(super) struct OrderbookResponse {
    pub(super) orderbook: IssTable<OrderbookLevelRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `candleborders`.
pub(super) struct CandleBordersResponse {
    pub(super) borders: IssTable<CandleBorderRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `candles`.
pub(super) struct CandlesResponse {
    pub(super) candles: IssTable<CandleRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `trades`.
pub(super) struct TradesResponse {
    pub(super) trades: IssTable<TradeRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint-а `securities/{secid}` (таблица `boards`).
pub(super) struct SecurityBoardsResponse {
    pub(super) boards: IssTable<SecurityBoardRow>,
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
/// Ответ объединённого endpoint-а `securities + marketdata`.
pub(super) struct BoardSecuritySnapshotsResponse {
    pub(super) securities: IssTable<BoardSecurityRow>,
    pub(super) marketdata: IssTable<BoardMarketDataRow>,
}

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `securities` для snapshot-режима.
pub(super) struct BoardSecurityRow(pub(super) String, pub(super) Option<i64>);

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `marketdata` для snapshot-режима.
pub(super) struct BoardMarketDataRow(pub(super) String, pub(super) Option<f64>);

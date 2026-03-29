//! Wire-структуры верхнего уровня для JSON-ответов ISS.
//!
//! Эти типы точно повторяют форму JSON, которую возвращает API:
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
/// Ответ endpoint `indices`.
pub(super) struct IndexesResponse {
    pub(super) indices: IssTable<IndexRow>,
}

#[cfg(feature = "history")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint `history/.../dates`.
pub(super) struct HistoryDatesResponse {
    pub(super) dates: IssTable<HistoryDatesRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `turnovers`.
pub(super) struct TurnoversResponse {
    pub(super) turnovers: IssTable<TurnoverRow>,
}

#[cfg(feature = "news")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint `sitenews`.
pub(super) struct SiteNewsResponse {
    pub(super) sitenews: IssTable<SiteNewsRow>,
}

#[cfg(feature = "news")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint `events`.
pub(super) struct EventsResponse {
    pub(super) events: IssTable<EventRow>,
}

#[cfg(feature = "history")]
#[derive(Debug, Deserialize)]
/// Ответ endpoint `history`.
pub(super) struct HistoryResponse {
    pub(super) history: IssTable<HistoryRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `secstats`.
pub(super) struct SecStatsResponse {
    pub(super) secstats: IssTable<SecStatRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `index_analytics`.
pub(super) struct IndexAnalyticsResponse {
    pub(super) analytics: IssTable<IndexAnalyticsRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `engines`.
pub(super) struct EnginesResponse {
    pub(super) engines: IssTable<EngineRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `markets`.
pub(super) struct MarketsResponse {
    pub(super) markets: IssTable<MarketRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `boards`.
pub(super) struct BoardsResponse {
    pub(super) boards: IssTable<BoardRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `securities`.
pub(super) struct SecuritiesResponse {
    pub(super) securities: IssTable<SecurityRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `orderbook`.
pub(super) struct OrderbookResponse {
    pub(super) orderbook: IssTable<OrderbookLevelRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `candleborders`.
pub(super) struct CandleBordersResponse {
    pub(super) borders: IssTable<CandleBorderRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `candles`.
pub(super) struct CandlesResponse {
    pub(super) candles: IssTable<CandleRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `trades`.
pub(super) struct TradesResponse {
    pub(super) trades: IssTable<TradeRow>,
}

#[derive(Debug, Deserialize)]
/// Ответ endpoint `securities/{secid}` для таблицы `boards`.
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

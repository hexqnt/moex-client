//! Преобразование wire-строк ISS в строгие доменные модели.
//!
//! Каждый конвертер сохраняет номер исходной строки (`row`) для структурированных
//! ошибок уровня [`MoexError`].

use crate::models::{
    Board, BoardRow, Candle, CandleBorder, CandleBorderRow, CandleRow, Engine, EngineRow, Index,
    IndexAnalytics, IndexAnalyticsRow, IndexRow, Market, MarketRow, OrderbookLevel,
    OrderbookLevelRow, SecStat, SecStatRow, Security, SecurityBoard, SecurityRow, Trade, TradeRow,
    Turnover, TurnoverRow,
};
#[cfg(feature = "news")]
use crate::models::{Event, EventRow, SiteNews, SiteNewsRow};
#[cfg(feature = "history")]
use crate::models::{HistoryDates, HistoryDatesRow, HistoryRecord, HistoryRow};

use super::MoexError;
use super::wire::SecurityBoardRow;

macro_rules! row_converters {
    ($($(#[$attr:meta])* $name:ident: $wire:ty => $model:ty, $error:ident;)*) => {
        $(
            $(#[$attr])*
            pub(super) fn $name(rows: Vec<$wire>, endpoint: &str) -> Result<Vec<$model>, MoexError> {
                rows.into_iter()
                    .enumerate()
                    .map(|(row, wire)| {
                        <$model>::try_from(wire).map_err(|source| MoexError::$error {
                            endpoint: endpoint.into(),
                            row,
                            source,
                        })
                    })
                    .collect()
            }
        )*
    };
}

row_converters! {
    convert_index_rows: IndexRow => Index, InvalidIndex;
    #[cfg(feature = "history")]
    convert_history_dates_rows: HistoryDatesRow => HistoryDates, InvalidHistoryDates;
    #[cfg(feature = "history")]
    convert_history_rows: HistoryRow => HistoryRecord, InvalidHistory;
    convert_turnover_rows: TurnoverRow => Turnover, InvalidTurnover;
    #[cfg(feature = "news")]
    convert_sitenews_rows: SiteNewsRow => SiteNews, InvalidSiteNews;
    #[cfg(feature = "news")]
    convert_event_rows: EventRow => Event, InvalidEvent;
    convert_secstats_rows: SecStatRow => SecStat, InvalidSecStat;
    convert_index_analytics_rows: IndexAnalyticsRow => IndexAnalytics, InvalidIndexAnalytics;
    convert_engine_rows: EngineRow => Engine, InvalidEngine;
    convert_market_rows: MarketRow => Market, InvalidMarket;
    convert_board_rows: BoardRow => Board, InvalidBoard;
    convert_security_rows: SecurityRow => Security, InvalidSecurity;
    convert_orderbook_rows: OrderbookLevelRow => OrderbookLevel, InvalidOrderbook;
    convert_candle_border_rows: CandleBorderRow => CandleBorder, InvalidCandleBorder;
    convert_candle_rows: CandleRow => Candle, InvalidCandle;
    convert_trade_rows: TradeRow => Trade, InvalidTrade;
}

pub(super) fn convert_security_board_rows(
    rows: Vec<SecurityBoardRow>,
    endpoint: &str,
) -> Result<Vec<SecurityBoard>, MoexError> {
    rows.into_iter()
        .enumerate()
        // Явно распаковываем tuple-строку, чтобы не терять соответствие с wire-полями.
        .map(
            |(row, SecurityBoardRow(engine, market, boardid, is_primary))| {
                SecurityBoard::try_new(engine, market, boardid, is_primary).map_err(|source| {
                    MoexError::InvalidSecurityBoard {
                        endpoint: endpoint.to_owned().into_boxed_str(),
                        row,
                        source,
                    }
                })
            },
        )
        .collect()
}

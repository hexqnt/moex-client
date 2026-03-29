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

pub(super) fn convert_index_rows(
    rows: Vec<IndexRow>,
    endpoint: &str,
) -> Result<Vec<Index>, MoexError> {
    rows.into_iter()
        .enumerate()
        // Индекс из `enumerate` нужен для точной диагностики проблемной строки ISS.
        .map(|(row, wire)| {
            Index::try_from(wire).map_err(|source| MoexError::InvalidIndex {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

#[cfg(feature = "history")]
pub(super) fn convert_history_dates_rows(
    rows: Vec<HistoryDatesRow>,
    endpoint: &str,
) -> Result<Vec<HistoryDates>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            HistoryDates::try_from(wire).map_err(|source| MoexError::InvalidHistoryDates {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

#[cfg(feature = "history")]
pub(super) fn convert_history_rows(
    rows: Vec<HistoryRow>,
    endpoint: &str,
) -> Result<Vec<HistoryRecord>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            HistoryRecord::try_from(wire).map_err(|source| MoexError::InvalidHistory {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_turnover_rows(
    rows: Vec<TurnoverRow>,
    endpoint: &str,
) -> Result<Vec<Turnover>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Turnover::try_from(wire).map_err(|source| MoexError::InvalidTurnover {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

#[cfg(feature = "news")]
pub(super) fn convert_sitenews_rows(
    rows: Vec<SiteNewsRow>,
    endpoint: &str,
) -> Result<Vec<SiteNews>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            SiteNews::try_from(wire).map_err(|source| MoexError::InvalidSiteNews {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

#[cfg(feature = "news")]
pub(super) fn convert_event_rows(
    rows: Vec<EventRow>,
    endpoint: &str,
) -> Result<Vec<Event>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Event::try_from(wire).map_err(|source| MoexError::InvalidEvent {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_secstats_rows(
    rows: Vec<SecStatRow>,
    endpoint: &str,
) -> Result<Vec<SecStat>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            SecStat::try_from(wire).map_err(|source| MoexError::InvalidSecStat {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_index_analytics_rows(
    rows: Vec<IndexAnalyticsRow>,
    endpoint: &str,
) -> Result<Vec<IndexAnalytics>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            IndexAnalytics::try_from(wire).map_err(|source| MoexError::InvalidIndexAnalytics {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_engine_rows(
    rows: Vec<EngineRow>,
    endpoint: &str,
) -> Result<Vec<Engine>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Engine::try_from(wire).map_err(|source| MoexError::InvalidEngine {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_market_rows(
    rows: Vec<MarketRow>,
    endpoint: &str,
) -> Result<Vec<Market>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Market::try_from(wire).map_err(|source| MoexError::InvalidMarket {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_board_rows(
    rows: Vec<BoardRow>,
    endpoint: &str,
) -> Result<Vec<Board>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Board::try_from(wire).map_err(|source| MoexError::InvalidBoard {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_security_rows(
    rows: Vec<SecurityRow>,
    endpoint: &str,
) -> Result<Vec<Security>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Security::try_from(wire).map_err(|source| MoexError::InvalidSecurity {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
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

pub(super) fn convert_orderbook_rows(
    rows: Vec<OrderbookLevelRow>,
    endpoint: &str,
) -> Result<Vec<OrderbookLevel>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            OrderbookLevel::try_from(wire).map_err(|source| MoexError::InvalidOrderbook {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_candle_border_rows(
    rows: Vec<CandleBorderRow>,
    endpoint: &str,
) -> Result<Vec<CandleBorder>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            CandleBorder::try_from(wire).map_err(|source| MoexError::InvalidCandleBorder {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_candle_rows(
    rows: Vec<CandleRow>,
    endpoint: &str,
) -> Result<Vec<Candle>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Candle::try_from(wire).map_err(|source| MoexError::InvalidCandle {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

pub(super) fn convert_trade_rows(
    rows: Vec<TradeRow>,
    endpoint: &str,
) -> Result<Vec<Trade>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| {
            Trade::try_from(wire).map_err(|source| MoexError::InvalidTrade {
                endpoint: endpoint.to_owned().into_boxed_str(),
                row,
                source,
            })
        })
        .collect()
}

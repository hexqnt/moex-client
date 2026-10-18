//! Типизированные декодеры ISS и объединение снимков инструментов.

mod raw;

use std::collections::HashMap;

use crate::models::{
    Board, BoardRow, Candle, CandleBorder, CandleBorderRow, CandleRow, Engine, EngineRow, Index,
    IndexAnalytics, IndexAnalyticsRow, IndexRow, Market, MarketRow, OrderbookLevel,
    OrderbookLevelRow, ParseSecuritySnapshotError, SecId, SecStat, SecStatRow, Security,
    SecurityBoard, SecurityRow, SecuritySnapshot, Trade, TradeRow, Turnover, TurnoverRow,
};
#[cfg(feature = "news")]
use crate::models::{Event, EventRow, SiteNews, SiteNewsRow};
#[cfg(feature = "history")]
use crate::models::{HistoryDates, HistoryDatesRow, HistoryRecord};

use super::MoexError;
use super::wire::*;

pub use raw::{RawTableView, RawTables};
pub(super) use raw::{
    decode_raw_table_rows_json_with_endpoint, decode_raw_table_view_json_with_endpoint,
    decode_raw_tables_json_with_endpoint,
};

fn parse_json<'de, T: serde::Deserialize<'de>>(
    payload: &'de str,
    endpoint: &str,
) -> Result<T, MoexError> {
    serde_json::from_str(payload).map_err(|source| MoexError::Decode {
        endpoint: endpoint.into(),
        source,
    })
}

#[inline]
fn convert_rows<Wire, Model: TryFrom<Wire>>(
    rows: Vec<Wire>,
    invalid_row: impl Fn(usize, Model::Error) -> MoexError,
) -> Result<Vec<Model>, MoexError> {
    rows.into_iter()
        .enumerate()
        .map(|(row, wire)| Model::try_from(wire).map_err(|source| invalid_row(row, source)))
        .collect()
}

macro_rules! table_decoders {
    ($($(#[$attr:meta])* $name:ident: $response:ident.$table:ident, $wire:ty => $model:ty, $error:ident;)*) => {
        $(
            $(#[$attr])*
            pub(super) fn $name(payload: &str, endpoint: &str) -> Result<Vec<$model>, MoexError> {
                // Derive сохраняет диагностику отсутствующих и повторных блоков serde.
                #[derive(serde::Deserialize)]
                struct $response {
                    $table: IssTable<$wire>,
                }

                let payload: $response = parse_json(payload, endpoint)?;
                convert_rows(payload.$table.data, |row, source| MoexError::$error {
                    endpoint: endpoint.into(),
                    row,
                    source,
                })
            }
        )*
    };
}

table_decoders! {
    decode_indexes_json_with_endpoint: IndexesResponse.indices, IndexRow => Index, InvalidIndex;
    decode_engines_json_with_endpoint: EnginesResponse.engines, EngineRow => Engine, InvalidEngine;
    decode_markets_json_with_endpoint: MarketsResponse.markets, MarketRow => Market, InvalidMarket;
    decode_boards_json_with_endpoint: BoardsResponse.boards, BoardRow => Board, InvalidBoard;
    decode_security_boards_json_with_endpoint: SecurityBoardsResponse.boards, SecurityBoardRow => SecurityBoard, InvalidSecurityBoard;
    decode_securities_json_with_endpoint: SecuritiesResponse.securities, SecurityRow => Security, InvalidSecurity;
    decode_orderbook_json_with_endpoint: OrderbookResponse.orderbook, OrderbookLevelRow => OrderbookLevel, InvalidOrderbook;
    decode_candle_borders_json_with_endpoint: CandleBordersResponse.borders, CandleBorderRow => CandleBorder, InvalidCandleBorder;
    decode_candles_json_with_endpoint: CandlesResponse.candles, CandleRow => Candle, InvalidCandle;
    decode_trades_json_with_endpoint: TradesResponse.trades, TradeRow => Trade, InvalidTrade;
    decode_index_analytics_json_with_endpoint: IndexAnalyticsResponse.analytics, IndexAnalyticsRow => IndexAnalytics, InvalidIndexAnalytics;
    #[cfg(feature = "history")]
    decode_history_dates_json_with_endpoint: HistoryDatesResponse.dates, HistoryDatesRow => HistoryDates, InvalidHistoryDates;
    decode_turnovers_json_with_endpoint: TurnoversResponse.turnovers, TurnoverRow => Turnover, InvalidTurnover;
    #[cfg(feature = "news")]
    decode_sitenews_json_with_endpoint: SiteNewsResponse.sitenews, SiteNewsRow => SiteNews, InvalidSiteNews;
    #[cfg(feature = "news")]
    decode_events_json_with_endpoint: EventsResponse.events, EventRow => Event, InvalidEvent;
    decode_secstats_json_with_endpoint: SecStatsResponse.secstats, SecStatRow => SecStat, InvalidSecStat;
}

pub(super) fn decode_board_security_snapshots_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<SecuritySnapshot>, MoexError> {
    let payload: BoardSecuritySnapshotsResponse = parse_json(payload, endpoint)?;

    let mut last_by_secid: HashMap<SecId, (usize, Option<f64>)> =
        HashMap::with_capacity(payload.marketdata.data.len());
    for (row, BoardMarketDataRow(secid, last)) in payload.marketdata.data.into_iter().enumerate() {
        if let Some(last) = last
            && !last.is_finite()
        {
            return Err(MoexError::InvalidSecuritySnapshot {
                endpoint: endpoint.to_owned().into_boxed_str(),
                table: "marketdata",
                row,
                source: ParseSecuritySnapshotError::NonFiniteLast(last),
            });
        }
        let secid = parse_snapshot_secid(endpoint, "marketdata", row, secid)?;
        last_by_secid.insert(secid, (row, last));
    }

    // В худшем случае таблицы не пересекаются по SECID, поэтому итог содержит
    // сумму строк обеих таблиц. Резервируем её сразу, исключая рост Vec.
    let snapshot_capacity = payload
        .securities
        .data
        .len()
        .saturating_add(last_by_secid.len());
    let mut snapshots = Vec::with_capacity(snapshot_capacity);
    for (row, BoardSecurityRow(secid, lot_size_raw)) in
        payload.securities.data.into_iter().enumerate()
    {
        let secid = parse_snapshot_secid(endpoint, "securities", row, secid)?;
        let lot_size = SecuritySnapshot::parse_lot_size(lot_size_raw).map_err(|source| {
            MoexError::InvalidSecuritySnapshot {
                endpoint: endpoint.into(),
                table: "securities",
                row,
                source,
            }
        })?;
        let last = last_by_secid.remove(&secid).and_then(|(_, last)| last);

        let snapshot =
            SecuritySnapshot::try_from_parts(secid, lot_size, last).map_err(|source| {
                MoexError::InvalidSecuritySnapshot {
                    endpoint: endpoint.to_owned().into_boxed_str(),
                    table: "securities",
                    row,
                    source,
                }
            })?;
        snapshots.push(snapshot);
    }

    for (secid, (row, last)) in last_by_secid {
        let snapshot = SecuritySnapshot::try_from_parts(secid, None, last).map_err(|source| {
            MoexError::InvalidSecuritySnapshot {
                endpoint: endpoint.to_owned().into_boxed_str(),
                table: "marketdata",
                row,
                source,
            }
        })?;
        snapshots.push(snapshot);
    }

    Ok(snapshots)
}

#[cfg(feature = "history")]
pub(super) fn decode_history_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<HistoryRecord>, MoexError> {
    let rows: Vec<crate::models::HistoryRow> =
        decode_raw_table_rows_json_with_endpoint(payload, endpoint, "history")?;
    convert_rows(rows, |row, source| MoexError::InvalidHistory {
        endpoint: endpoint.into(),
        row,
        source,
    })
}

fn parse_snapshot_secid(
    endpoint: &str,
    table: &'static str,
    row: usize,
    secid: String,
) -> Result<SecId, MoexError> {
    SecId::try_from(secid).map_err(|source| MoexError::InvalidSecuritySnapshot {
        endpoint: endpoint.to_owned().into_boxed_str(),
        table,
        row,
        source: ParseSecuritySnapshotError::InvalidSecId(source),
    })
}

#[cfg(test)]
mod tests;

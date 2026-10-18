//! Типизированные декодеры ISS и объединение снимков инструментов.

mod raw;

use std::collections::HashMap;

use crate::models::{
    Board, Candle, CandleBorder, Engine, Index, IndexAnalytics, Market, OrderbookLevel,
    ParseSecuritySnapshotError, SecId, SecStat, Security, SecurityBoard, SecuritySnapshot, Trade,
    Turnover,
};
#[cfg(feature = "news")]
use crate::models::{Event, SiteNews};
#[cfg(feature = "history")]
use crate::models::{HistoryDates, HistoryRecord};

use super::MoexError;
use super::constants::*;
use super::convert::*;
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

macro_rules! table_decoders {
    ($($(#[$attr:meta])* $name:ident: $model:ty, $response:ident.$table:ident, $convert:ident;)*) => {
        $(
            $(#[$attr])*
            pub(super) fn $name(payload: &str, endpoint: &str) -> Result<Vec<$model>, MoexError> {
                let payload: $response = parse_json(payload, endpoint)?;
                $convert(payload.$table.data, endpoint)
            }
        )*
    };
}

table_decoders! {
    decode_markets_json_with_endpoint: Market, MarketsResponse.markets, convert_market_rows;
    decode_boards_json_with_endpoint: Board, BoardsResponse.boards, convert_board_rows;
    decode_security_boards_json_with_endpoint: SecurityBoard, SecurityBoardsResponse.boards, convert_security_board_rows;
    decode_securities_json_with_endpoint: Security, SecuritiesResponse.securities, convert_security_rows;
    decode_orderbook_json_with_endpoint: OrderbookLevel, OrderbookResponse.orderbook, convert_orderbook_rows;
    decode_candle_borders_json_with_endpoint: CandleBorder, CandleBordersResponse.borders, convert_candle_border_rows;
    decode_candles_json_with_endpoint: Candle, CandlesResponse.candles, convert_candle_rows;
    decode_trades_json_with_endpoint: Trade, TradesResponse.trades, convert_trade_rows;
    decode_index_analytics_json_with_endpoint: IndexAnalytics, IndexAnalyticsResponse.analytics, convert_index_analytics_rows;
    #[cfg(feature = "history")]
    decode_history_dates_json_with_endpoint: HistoryDates, HistoryDatesResponse.dates, convert_history_dates_rows;
    decode_turnovers_json_with_endpoint: Turnover, TurnoversResponse.turnovers, convert_turnover_rows;
    #[cfg(feature = "news")]
    decode_sitenews_json_with_endpoint: SiteNews, SiteNewsResponse.sitenews, convert_sitenews_rows;
    #[cfg(feature = "news")]
    decode_events_json_with_endpoint: Event, EventsResponse.events, convert_event_rows;
    decode_secstats_json_with_endpoint: SecStat, SecStatsResponse.secstats, convert_secstats_rows;
}

pub(super) fn decode_indexes_json_payload(payload: &str) -> Result<Vec<Index>, MoexError> {
    let payload: IndexesResponse = parse_json(payload, INDEXES_ENDPOINT)?;
    convert_index_rows(payload.indices.data, INDEXES_ENDPOINT)
}

pub(super) fn decode_engines_json_payload(payload: &str) -> Result<Vec<Engine>, MoexError> {
    let payload: EnginesResponse = parse_json(payload, ENGINES_ENDPOINT)?;
    convert_engine_rows(payload.engines.data, ENGINES_ENDPOINT)
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
        let lot_size = parse_snapshot_lot_size(endpoint, "securities", row, lot_size_raw)?;
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
    let rows = decode_raw_table_rows_json_with_endpoint(payload, endpoint, "history")?;
    convert_history_rows(rows, endpoint)
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

fn parse_snapshot_lot_size(
    endpoint: &str,
    table: &'static str,
    row: usize,
    lot_size: Option<i64>,
) -> Result<Option<u32>, MoexError> {
    match lot_size {
        None => Ok(None),
        Some(raw) if raw < 0 => Err(MoexError::InvalidSecuritySnapshot {
            endpoint: endpoint.to_owned().into_boxed_str(),
            table,
            row,
            source: ParseSecuritySnapshotError::NegativeLotSize(raw),
        }),
        Some(raw) => u32::try_from(raw)
            .map(Some)
            .map_err(|_| MoexError::InvalidSecuritySnapshot {
                endpoint: endpoint.to_owned().into_boxed_str(),
                table,
                row,
                source: ParseSecuritySnapshotError::LotSizeOutOfRange(raw),
            }),
    }
}

#[cfg(test)]
mod tests;

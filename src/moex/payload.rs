use std::borrow::Cow;
use std::collections::HashMap;

use serde::de::{
    DeserializeSeed, IgnoredAny, IntoDeserializer, MapAccess, Visitor,
    value::{MapDeserializer, StrDeserializer},
};
use serde_json::value::RawValue;

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

#[derive(Debug, serde::Deserialize)]
struct RawIssTableRowsPayload {
    columns: Vec<Box<str>>,
    data: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug)]
/// Разобранные top-level блоки ISS payload-а для декодирования нескольких таблиц.
///
/// Позволяет один раз разобрать payload и затем извлекать таблицы через
/// [`RawTables::take_rows`] без повторного разбора всего JSON-документа.
pub struct RawTables {
    endpoint: Box<str>,
    blocks: HashMap<Box<str>, Box<RawValue>>,
}

impl RawTables {
    /// Количество top-level блоков в payload-е.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Проверить, что payload не содержит top-level блоков.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Имена top-level блоков payload-а.
    ///
    /// Итератор отражает текущее состояние кэша: после `take_rows(...)`
    /// соответствующее имя больше не возвращается.
    pub fn table_names(&self) -> impl Iterator<Item = &str> {
        self.blocks.keys().map(Box::as_ref)
    }

    /// Декодировать строки выбранной таблицы и удалить её из кэша.
    ///
    /// Метод потребляет только выбранный top-level блок и не вызывает
    /// повторный разбор всего payload-а.
    pub fn take_rows<T>(&mut self, table: impl Into<String>) -> Result<Vec<T>, MoexError>
    where
        T: serde::de::DeserializeOwned,
    {
        let table: Box<str> = table.into().into_boxed_str();
        let raw_table =
            self.blocks
                .remove(table.as_ref())
                .ok_or_else(|| MoexError::MissingRawTable {
                    endpoint: self.endpoint.clone(),
                    table: table.clone(),
                })?;
        // Декодируем только выбранный блок, не затрагивая остальные таблицы.
        let table_payload =
            serde_json::from_str(raw_table.get()).map_err(|source| MoexError::Decode {
                endpoint: format!("{} (table={})", self.endpoint, table).into_boxed_str(),
                source,
            })?;
        decode_raw_table_rows_payload_with_context(
            table_payload,
            self.endpoint.as_ref(),
            table.as_ref(),
        )
    }
}

#[derive(Debug)]
/// Borrowed-представление ISS-таблицы без промежуточного `Value`.
pub struct RawTableView<'a> {
    columns: Vec<Cow<'a, str>>,
    data: Vec<Vec<&'a RawValue>>,
}

impl<'a> RawTableView<'a> {
    /// Колонки таблицы в исходном порядке.
    pub fn columns(&self) -> &[Cow<'a, str>] {
        &self.columns
    }

    /// Все строки таблицы в виде raw JSON-значений.
    pub fn rows(&self) -> &[Vec<&'a RawValue>] {
        &self.data
    }

    /// Количество строк в таблице.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Проверить, пуста ли таблица.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Вернуть индекс колонки по имени.
    pub fn column_index(&self, column: &str) -> Option<usize> {
        self.columns.iter().position(|name| name.as_ref() == column)
    }

    /// Вернуть raw JSON-значение по `row` и имени `column`.
    pub fn raw_value(&self, row: usize, column: &str) -> Option<&'a RawValue> {
        let column_index = self.column_index(column)?;
        self.data
            .get(row)
            .and_then(|values| values.get(column_index))
            .copied()
    }

    /// Десериализовать значение колонки в пользовательский тип.
    ///
    /// Тип `T` может заимствовать данные из payload-а.
    pub fn deserialize_value<T>(
        &self,
        row: usize,
        column: &str,
    ) -> Result<Option<T>, serde_json::Error>
    where
        T: serde::de::Deserialize<'a>,
    {
        self.raw_value(row, column)
            .map(|raw| serde_json::from_str(raw.get()))
            .transpose()
    }
}

struct RawIssTableRowsSeed<'a> {
    table: &'a str,
}

impl<'de> DeserializeSeed<'de> for RawIssTableRowsSeed<'_> {
    type Value = Option<RawIssTableRowsPayload>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RawIssTableRowsVisitor { table: self.table })
    }
}

struct RawIssTableRowsVisitor<'a> {
    table: &'a str,
}

impl<'de> Visitor<'de> for RawIssTableRowsVisitor<'_> {
    type Value = Option<RawIssTableRowsPayload>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ISS JSON object with table blocks")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut selected = None;

        while let Some(key) = map.next_key::<Cow<'de, str>>()? {
            if key == self.table && selected.is_none() {
                selected = Some(map.next_value::<RawIssTableRowsPayload>()?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }

        Ok(selected)
    }
}

fn decode_single_raw_table_payload(
    payload: &str,
    table: &str,
) -> Result<Option<RawIssTableRowsPayload>, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(payload);
    RawIssTableRowsSeed { table }.deserialize(&mut deserializer)
}

fn decode_top_level_raw_blocks(
    payload: &str,
) -> Result<HashMap<Box<str>, Box<RawValue>>, serde_json::Error> {
    // Сохраняем каждый top-level блок как `RawValue`, чтобы декодировать
    // конкретные таблицы позже по запросу пользователя.
    serde_json::from_str(payload)
}

#[derive(Debug, serde::Deserialize)]
struct BorrowedRawIssTableRowsPayload<'a> {
    #[serde(borrow)]
    columns: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    data: Vec<Vec<&'a RawValue>>,
}

struct BorrowedRawIssTableRowsSeed<'a> {
    table: &'a str,
}

impl<'de> DeserializeSeed<'de> for BorrowedRawIssTableRowsSeed<'_> {
    type Value = Option<BorrowedRawIssTableRowsPayload<'de>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(BorrowedRawIssTableRowsVisitor { table: self.table })
    }
}

struct BorrowedRawIssTableRowsVisitor<'a> {
    table: &'a str,
}

impl<'de> Visitor<'de> for BorrowedRawIssTableRowsVisitor<'_> {
    type Value = Option<BorrowedRawIssTableRowsPayload<'de>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ISS JSON object with table blocks")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut selected = None;

        while let Some(key) = map.next_key::<Cow<'de, str>>()? {
            if key == self.table && selected.is_none() {
                selected = Some(map.next_value::<BorrowedRawIssTableRowsPayload<'de>>()?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }

        Ok(selected)
    }
}

fn decode_single_raw_table_payload_borrowed<'a>(
    payload: &'a str,
    table: &str,
) -> Result<Option<BorrowedRawIssTableRowsPayload<'a>>, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(payload);
    BorrowedRawIssTableRowsSeed { table }.deserialize(&mut deserializer)
}

fn decode_raw_table_row<T>(
    columns: &[Box<str>],
    values: Vec<serde_json::Value>,
) -> Result<T, serde_json::Error>
where
    T: serde::de::DeserializeOwned,
{
    let entries = columns
        .iter()
        .map(Box::as_ref)
        .zip(values)
        .map(|(column, value)| {
            (
                StrDeserializer::<serde_json::Error>::new(column),
                value.into_deserializer(),
            )
        });
    let deserializer = MapDeserializer::new(entries);
    <T as serde::Deserialize>::deserialize(deserializer)
}

fn decode_raw_table_rows_payload_with_context<T>(
    table_payload: RawIssTableRowsPayload,
    endpoint: &str,
    table: &str,
) -> Result<Vec<T>, MoexError>
where
    T: serde::de::DeserializeOwned,
{
    let endpoint: Box<str> = endpoint.to_owned().into_boxed_str();
    let table: Box<str> = table.to_owned().into_boxed_str();
    let RawIssTableRowsPayload { columns, data } = table_payload;
    let expected_width = columns.len();
    let mut rows = Vec::with_capacity(data.len());

    for (row, values) in data.into_iter().enumerate() {
        let actual_width = values.len();
        if actual_width != expected_width {
            return Err(MoexError::InvalidRawTableRowWidth {
                endpoint: endpoint.clone(),
                table: table.clone(),
                row,
                expected: expected_width,
                actual: actual_width,
            });
        }

        let decoded = decode_raw_table_row::<T>(&columns, values).map_err(|source| {
            MoexError::InvalidRawTableRow {
                endpoint: endpoint.clone(),
                table: table.clone(),
                row,
                source,
            }
        })?;
        rows.push(decoded);
    }

    Ok(rows)
}

pub(super) fn decode_raw_table_rows_json_with_endpoint<T>(
    payload: &str,
    endpoint: &str,
    table: &str,
) -> Result<Vec<T>, MoexError>
where
    T: serde::de::DeserializeOwned,
{
    let endpoint = endpoint.to_owned().into_boxed_str();
    let table = table.to_owned().into_boxed_str();

    let table_payload = decode_single_raw_table_payload(payload, table.as_ref())
        .map_err(|source| MoexError::Decode {
            endpoint: endpoint.clone(),
            source,
        })?
        .ok_or_else(|| MoexError::MissingRawTable {
            endpoint: endpoint.clone(),
            table: table.clone(),
        })?;
    decode_raw_table_rows_payload_with_context(table_payload, endpoint.as_ref(), table.as_ref())
}

pub(super) fn decode_raw_tables_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<RawTables, MoexError> {
    let endpoint = endpoint.to_owned().into_boxed_str();
    let raw_blocks = decode_top_level_raw_blocks(payload).map_err(|source| MoexError::Decode {
        endpoint: endpoint.clone(),
        source,
    })?;
    Ok(RawTables {
        endpoint,
        blocks: raw_blocks,
    })
}

pub(super) fn decode_raw_table_view_json_with_endpoint<'a>(
    payload: &'a str,
    endpoint: &str,
    table: &str,
) -> Result<RawTableView<'a>, MoexError> {
    let endpoint = endpoint.to_owned().into_boxed_str();
    let table = table.to_owned().into_boxed_str();

    let table_payload = decode_single_raw_table_payload_borrowed(payload, table.as_ref())
        .map_err(|source| MoexError::Decode {
            endpoint: endpoint.clone(),
            source,
        })?
        .ok_or_else(|| MoexError::MissingRawTable {
            endpoint: endpoint.clone(),
            table: table.clone(),
        })?;

    let expected_width = table_payload.columns.len();
    for (row, values) in table_payload.data.iter().enumerate() {
        let actual_width = values.len();
        if actual_width != expected_width {
            return Err(MoexError::InvalidRawTableRowWidth {
                endpoint: endpoint.clone(),
                table: table.clone(),
                row,
                expected: expected_width,
                actual: actual_width,
            });
        }
    }

    Ok(RawTableView {
        columns: table_payload.columns,
        data: table_payload.data,
    })
}

pub(super) fn decode_indexes_json_payload(payload: &str) -> Result<Vec<Index>, MoexError> {
    let payload: IndexesResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: INDEXES_ENDPOINT.to_owned().into_boxed_str(),
            source,
        })?;
    convert_index_rows(payload.indices.data, INDEXES_ENDPOINT)
}

pub(super) fn decode_engines_json_payload(payload: &str) -> Result<Vec<Engine>, MoexError> {
    let payload: EnginesResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: ENGINES_ENDPOINT.to_owned().into_boxed_str(),
            source,
        })?;
    convert_engine_rows(payload.engines.data, ENGINES_ENDPOINT)
}

pub(super) fn decode_markets_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<Market>, MoexError> {
    let payload: MarketsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_market_rows(payload.markets.data, endpoint)
}

pub(super) fn decode_boards_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<Board>, MoexError> {
    let payload: BoardsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_board_rows(payload.boards.data, endpoint)
}

pub(super) fn decode_security_boards_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<SecurityBoard>, MoexError> {
    let payload: SecurityBoardsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_security_board_rows(payload.boards.data, endpoint)
}

pub(super) fn decode_securities_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<Security>, MoexError> {
    let payload: SecuritiesResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_security_rows(payload.securities.data, endpoint)
}

pub(super) fn decode_board_security_snapshots_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<SecuritySnapshot>, MoexError> {
    let payload: BoardSecuritySnapshotsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;

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

    let mut snapshots = Vec::with_capacity(payload.securities.data.len().max(last_by_secid.len()));
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

pub(super) fn decode_orderbook_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<OrderbookLevel>, MoexError> {
    let payload: OrderbookResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_orderbook_rows(payload.orderbook.data, endpoint)
}

pub(super) fn decode_candle_borders_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<CandleBorder>, MoexError> {
    let payload: CandleBordersResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_candle_border_rows(payload.borders.data, endpoint)
}

pub(super) fn decode_candles_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<Candle>, MoexError> {
    let payload: CandlesResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_candle_rows(payload.candles.data, endpoint)
}

pub(super) fn decode_trades_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<Trade>, MoexError> {
    let payload: TradesResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_trade_rows(payload.trades.data, endpoint)
}

pub(super) fn decode_index_analytics_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<IndexAnalytics>, MoexError> {
    let payload: IndexAnalyticsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_index_analytics_rows(payload.analytics.data, endpoint)
}

#[cfg(feature = "history")]
pub(super) fn decode_history_dates_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<HistoryDates>, MoexError> {
    let payload: HistoryDatesResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_history_dates_rows(payload.dates.data, endpoint)
}

#[cfg(feature = "history")]
pub(super) fn decode_history_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<HistoryRecord>, MoexError> {
    let payload: HistoryResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_history_rows(payload.history.data, endpoint)
}

pub(super) fn decode_turnovers_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<Turnover>, MoexError> {
    let payload: TurnoversResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_turnover_rows(payload.turnovers.data, endpoint)
}

#[cfg(feature = "news")]
pub(super) fn decode_sitenews_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<SiteNews>, MoexError> {
    let payload: SiteNewsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_sitenews_rows(payload.sitenews.data, endpoint)
}

#[cfg(feature = "news")]
pub(super) fn decode_events_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<Event>, MoexError> {
    let payload: EventsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_event_rows(payload.events.data, endpoint)
}

pub(super) fn decode_secstats_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<Vec<SecStat>, MoexError> {
    let payload: SecStatsResponse =
        serde_json::from_str(payload).map_err(|source| MoexError::Decode {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;
    convert_secstats_rows(payload.secstats.data, endpoint)
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

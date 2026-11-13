//! Пользовательские таблицы ISS: owned-строки, borrowed-ячейки и отложенный разбор блоков.

use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;

use serde::de::{
    DeserializeSeed, IgnoredAny, IntoDeserializer, MapAccess, Visitor,
    value::{MapDeserializer, StrDeserializer},
};
use serde_json::value::RawValue;

use super::{MoexError, parse_json};

#[derive(Debug, serde::Deserialize)]
struct RawIssTableRowsPayload {
    columns: Vec<Box<str>>,
    data: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug)]
/// Разобранные верхнеуровневые блоки ISS payload-а для декодирования нескольких таблиц.
///
/// Позволяет один раз разобрать payload и затем извлекать таблицы через
/// [`RawTables::take_rows`] без повторного разбора всего JSON-документа.
pub struct RawTables {
    endpoint: Box<str>,
    blocks: HashMap<Box<str>, Box<RawValue>>,
}

impl RawTables {
    /// Количество верхнеуровневых блоков в payload-е.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Проверить, что payload не содержит верхнеуровневых блоков.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Имена верхнеуровневых блоков payload-а.
    ///
    /// Итератор отражает текущее состояние кэша: после `take_rows(...)`
    /// соответствующее имя больше не возвращается.
    pub fn table_names(&self) -> impl Iterator<Item = &str> {
        self.blocks.keys().map(Box::as_ref)
    }

    /// Декодировать строки выбранной таблицы и удалить её из кэша.
    ///
    /// Метод потребляет только выбранный верхнеуровневый блок и не вызывает
    /// повторный разбор всего payload-а.
    pub fn take_rows<T>(&mut self, table: impl AsRef<str>) -> Result<Vec<T>, MoexError>
    where
        T: serde::de::DeserializeOwned,
    {
        let table = table.as_ref();
        let raw_table = self
            .blocks
            .remove(table)
            .ok_or_else(|| MoexError::MissingRawTable {
                endpoint: self.endpoint.clone(),
                table: table.into(),
            })?;
        // Декодируем только выбранный блок, не затрагивая остальные таблицы.
        let table_payload =
            serde_json::from_str(raw_table.get()).map_err(|source| MoexError::Decode {
                endpoint: format!("{} (table={})", self.endpoint, table).into_boxed_str(),
                source,
            })?;
        decode_raw_table_rows_payload_with_context(table_payload, self.endpoint.as_ref(), table)
    }
}

#[derive(Debug)]
/// Заимствованное представление ISS-таблицы без промежуточного `Value`.
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

#[derive(Debug, serde::Deserialize)]
struct BorrowedRawIssTableRowsPayload<'a> {
    #[serde(borrow)]
    columns: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    data: Vec<Vec<&'a RawValue>>,
}

struct SelectedTable<'a, T> {
    table: &'a str,
    marker: PhantomData<fn() -> T>,
}

impl<'de, T: serde::Deserialize<'de>> DeserializeSeed<'de> for SelectedTable<'_, T> {
    type Value = Option<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for SelectedTable<'_, T> {
    type Value = Option<T>;

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
                selected = Some(map.next_value::<T>()?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(selected)
    }
}

fn decode_single_raw_table_payload<'de, T: serde::Deserialize<'de>>(
    payload: &'de str,
    endpoint: &str,
    table: &str,
) -> Result<T, MoexError> {
    let mut deserializer = serde_json::Deserializer::from_str(payload);
    SelectedTable {
        table,
        marker: PhantomData,
    }
    .deserialize(&mut deserializer)
    .map_err(|source| MoexError::Decode {
        endpoint: endpoint.into(),
        source,
    })?
    .ok_or_else(|| MoexError::MissingRawTable {
        endpoint: endpoint.into(),
        table: table.into(),
    })
}

#[inline]
fn check_row_width(
    expected: usize,
    actual: usize,
    endpoint: &str,
    table: &str,
    row: usize,
) -> Result<(), MoexError> {
    if actual != expected {
        return Err(MoexError::InvalidRawTableRowWidth {
            endpoint: endpoint.into(),
            table: table.into(),
            row,
            expected,
            actual,
        });
    }
    Ok(())
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
    let RawIssTableRowsPayload { columns, data } = table_payload;
    let expected_width = columns.len();
    let mut rows = Vec::with_capacity(data.len());

    for (row, values) in data.into_iter().enumerate() {
        check_row_width(expected_width, values.len(), endpoint, table, row)?;

        let decoded = decode_raw_table_row::<T>(&columns, values).map_err(|source| {
            MoexError::InvalidRawTableRow {
                endpoint: endpoint.into(),
                table: table.into(),
                row,
                source,
            }
        })?;
        rows.push(decoded);
    }

    Ok(rows)
}

pub(in crate::moex) fn decode_raw_table_rows_json_with_endpoint<T>(
    payload: &str,
    endpoint: &str,
    table: &str,
) -> Result<Vec<T>, MoexError>
where
    T: serde::de::DeserializeOwned,
{
    let table_payload = decode_single_raw_table_payload(payload, endpoint, table)?;
    decode_raw_table_rows_payload_with_context(table_payload, endpoint, table)
}

pub(in crate::moex) fn decode_raw_tables_json_with_endpoint(
    payload: &str,
    endpoint: &str,
) -> Result<RawTables, MoexError> {
    Ok(RawTables {
        blocks: parse_json(payload, endpoint)?,
        endpoint: endpoint.into(),
    })
}

pub(in crate::moex) fn decode_raw_table_view_json_with_endpoint<'a>(
    payload: &'a str,
    endpoint: &str,
    table: &str,
) -> Result<RawTableView<'a>, MoexError> {
    let table_payload: BorrowedRawIssTableRowsPayload<'a> =
        decode_single_raw_table_payload(payload, endpoint, table)?;

    let expected_width = table_payload.columns.len();
    for (row, values) in table_payload.data.iter().enumerate() {
        check_row_width(expected_width, values.len(), endpoint, table, row)?;
    }

    Ok(RawTableView {
        columns: table_payload.columns,
        data: table_payload.data,
    })
}

#[cfg(test)]
mod tests;

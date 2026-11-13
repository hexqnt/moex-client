//! Исторические торговые данные и параметры их выборки.

use chrono::NaiveDate;

use super::super::errors::{
    ParseHistoryDatesError, ParseHistoryQueryError, ParseHistoryRecordError,
};
use super::super::helpers::optional_non_negative;
use super::super::identifiers::{BoardId, SecId};
use super::super::wire::HistoryRow;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Диапазон доступных исторических дат (`history/.../dates`).
pub struct HistoryDates {
    from: NaiveDate,
    till: NaiveDate,
}

impl HistoryDates {
    /// Построить диапазон доступных исторических дат с проверкой порядка.
    pub fn try_new(from: NaiveDate, till: NaiveDate) -> Result<Self, ParseHistoryDatesError> {
        if from > till {
            return Err(ParseHistoryDatesError::InvalidDateRange { from, till });
        }
        Ok(Self { from, till })
    }

    /// Начальная дата доступной истории.
    pub fn from(&self) -> NaiveDate {
        self.from
    }

    /// Конечная дата доступной истории.
    pub fn till(&self) -> NaiveDate {
        self.till
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Строка исторических дневных торгов (`history`).
pub struct HistoryRecord {
    boardid: BoardId,
    tradedate: NaiveDate,
    secid: SecId,
    numtrades: Option<u64>,
    value: Option<f64>,
    open: Option<f64>,
    low: Option<f64>,
    high: Option<f64>,
    close: Option<f64>,
    volume: Option<u64>,
    duration_days: Option<f64>,
    yield_percent: Option<f64>,
}

impl HistoryRecord {
    /// Идентификатор режима торгов (`boardid`).
    pub fn boardid(&self) -> &BoardId {
        &self.boardid
    }

    /// Дата торговой сессии (`tradedate`).
    pub fn tradedate(&self) -> NaiveDate {
        self.tradedate
    }

    /// Идентификатор инструмента (`secid`).
    pub fn secid(&self) -> &SecId {
        &self.secid
    }

    /// Количество сделок (`numtrades`).
    pub fn numtrades(&self) -> Option<u64> {
        self.numtrades
    }

    /// Оборот в денежном выражении (`value`).
    pub fn value(&self) -> Option<f64> {
        self.value
    }

    /// Цена открытия (`open`).
    pub fn open(&self) -> Option<f64> {
        self.open
    }

    /// Минимальная цена (`low`).
    pub fn low(&self) -> Option<f64> {
        self.low
    }

    /// Максимальная цена (`high`).
    pub fn high(&self) -> Option<f64> {
        self.high
    }

    /// Цена закрытия (`close`).
    pub fn close(&self) -> Option<f64> {
        self.close
    }

    /// Объём торгов (`volume`).
    pub fn volume(&self) -> Option<u64> {
        self.volume
    }

    /// Дюрация в днях (`DURATION`), если поле присутствует в истории этого рынка.
    ///
    /// ISS использует для `DURATION` как целый, так и дробный wire-тип в зависимости
    /// от рынка, поэтому доменная модель хранит значение как `f64`.
    pub fn duration_days(&self) -> Option<f64> {
        self.duration_days
    }

    /// Доходность в процентах (`YIELD`), если поле присутствует в истории этого рынка.
    pub fn yield_percent(&self) -> Option<f64> {
        self.yield_percent
    }
}

impl TryFrom<HistoryRow> for HistoryRecord {
    type Error = ParseHistoryRecordError;

    fn try_from(row: HistoryRow) -> Result<Self, Self::Error> {
        let HistoryRow {
            boardid,
            tradedate,
            secid,
            numtrades,
            value,
            open,
            low,
            high,
            close,
            volume,
            duration_days,
            yield_percent,
        } = row;

        let boardid = BoardId::try_from(boardid)?;
        let secid = SecId::try_from(secid)?;

        let numtrades =
            optional_non_negative(numtrades, ParseHistoryRecordError::NegativeNumTrades)?;

        let volume = optional_non_negative(volume, ParseHistoryRecordError::NegativeVolume)?;

        Ok(Self {
            boardid,
            tradedate,
            secid,
            numtrades,
            value,
            open,
            low,
            high,
            close,
            volume,
            duration_days,
            yield_percent,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Строгие параметры запроса истории ISS с инвариантом `from <= till`.
#[must_use]
pub struct HistoryQuery {
    from: Option<NaiveDate>,
    till: Option<NaiveDate>,
}

impl HistoryQuery {
    /// Построить запрос истории с проверкой инварианта `from <= till`.
    pub fn try_new(
        from: Option<NaiveDate>,
        till: Option<NaiveDate>,
    ) -> Result<Self, ParseHistoryQueryError> {
        if let (Some(from), Some(till)) = (from, till)
            && from > till
        {
            return Err(ParseHistoryQueryError::InvalidDateRange { from, till });
        }

        Ok(Self { from, till })
    }

    /// Дата начала выборки (`from`).
    pub fn from(&self) -> Option<NaiveDate> {
        self.from
    }

    /// Дата окончания выборки (`till`).
    pub fn till(&self) -> Option<NaiveDate> {
        self.till
    }

    /// Вернуть копию запроса с новым `from`.
    pub fn with_from(self, from: NaiveDate) -> Result<Self, ParseHistoryQueryError> {
        Self::try_new(Some(from), self.till)
    }

    /// Вернуть копию запроса с новым `till`.
    pub fn with_till(self, till: NaiveDate) -> Result<Self, ParseHistoryQueryError> {
        Self::try_new(self.from, Some(till))
    }
}

tuple_row_conversion!(HistoryDatesRow => HistoryDates, ParseHistoryDatesError; 0, 1);

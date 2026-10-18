//! Свечи, их интервалы, доступные границы и параметры запроса.

use chrono::NaiveDateTime;

use super::super::errors::{
    ParseCandleBorderError, ParseCandleError, ParseCandleIntervalError, ParseCandleQueryError,
};
use super::super::helpers::{non_negative_u32, optional_non_negative};
use super::super::wire::CandleRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Интервалы свечей в терминах ISS.
pub enum CandleInterval {
    /// 1 минута.
    Minute1,
    /// 10 минут.
    Minute10,
    /// 1 час.
    Hour1,
    /// 1 день.
    Day1,
    /// 1 неделя.
    Week1,
    /// 1 месяц.
    Month1,
    /// 1 квартал.
    Quarter1,
}

impl CandleInterval {
    /// Вернуть строковый код интервала для query-параметра `interval`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minute1 => "1",
            Self::Minute10 => "10",
            Self::Hour1 => "60",
            Self::Day1 => "24",
            Self::Week1 => "7",
            Self::Month1 => "31",
            Self::Quarter1 => "4",
        }
    }
}

impl TryFrom<i64> for CandleInterval {
    type Error = ParseCandleIntervalError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Minute1),
            10 => Ok(Self::Minute10),
            60 => Ok(Self::Hour1),
            24 => Ok(Self::Day1),
            7 => Ok(Self::Week1),
            31 => Ok(Self::Month1),
            4 => Ok(Self::Quarter1),
            other => Err(ParseCandleIntervalError::InvalidCode(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Доступные границы свечных данных (`candleborders`).
pub struct CandleBorder {
    begin: NaiveDateTime,
    end: NaiveDateTime,
    interval: CandleInterval,
    board_group_id: u32,
}

impl CandleBorder {
    /// Построить границы доступных свечей из wire-значений ISS.
    pub fn try_new(
        begin: NaiveDateTime,
        end: NaiveDateTime,
        interval: i64,
        board_group_id: i64,
    ) -> Result<Self, ParseCandleBorderError> {
        if begin > end {
            return Err(ParseCandleBorderError::InvalidDateRange { begin, end });
        }

        let interval = CandleInterval::try_from(interval)?;
        let board_group_id = non_negative_u32(
            board_group_id,
            ParseCandleBorderError::NegativeBoardGroupId,
            ParseCandleBorderError::BoardGroupIdOutOfRange,
        )?;

        Ok(Self {
            begin,
            end,
            interval,
            board_group_id,
        })
    }

    /// Начало доступного диапазона.
    pub fn begin(&self) -> NaiveDateTime {
        self.begin
    }

    /// Конец доступного диапазона.
    pub fn end(&self) -> NaiveDateTime {
        self.end
    }

    /// Интервал свечей.
    pub fn interval(&self) -> CandleInterval {
        self.interval
    }

    /// Идентификатор группы режимов торгов.
    pub fn board_group_id(&self) -> u32 {
        self.board_group_id
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Строгие параметры запроса свечей ISS с инвариантом `from <= till`.
#[must_use]
pub struct CandleQuery {
    from: Option<NaiveDateTime>,
    till: Option<NaiveDateTime>,
    interval: Option<CandleInterval>,
}

impl CandleQuery {
    /// Построить запрос свечей с проверкой инварианта `from <= till`.
    pub fn try_new(
        from: Option<NaiveDateTime>,
        till: Option<NaiveDateTime>,
        interval: Option<CandleInterval>,
    ) -> Result<Self, ParseCandleQueryError> {
        if let (Some(from), Some(till)) = (from, till)
            && from > till
        {
            return Err(ParseCandleQueryError::InvalidDateRange { from, till });
        }

        Ok(Self {
            from,
            till,
            interval,
        })
    }

    /// Дата и время начала выборки (`from`).
    pub fn from(&self) -> Option<NaiveDateTime> {
        self.from
    }

    /// Дата и время окончания выборки (`till`).
    pub fn till(&self) -> Option<NaiveDateTime> {
        self.till
    }

    /// Интервал свечей (`interval`).
    pub fn interval(&self) -> Option<CandleInterval> {
        self.interval
    }

    /// Вернуть копию запроса с новым `from`.
    pub fn with_from(self, from: NaiveDateTime) -> Result<Self, ParseCandleQueryError> {
        Self::try_new(Some(from), self.till, self.interval)
    }

    /// Вернуть копию запроса с новым `till`.
    pub fn with_till(self, till: NaiveDateTime) -> Result<Self, ParseCandleQueryError> {
        Self::try_new(self.from, Some(till), self.interval)
    }

    /// Вернуть копию запроса с новым интервалом свечей.
    pub fn with_interval(mut self, interval: CandleInterval) -> Self {
        self.interval = Some(interval);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Свеча торгового инструмента (`candles`).
pub struct Candle {
    begin: NaiveDateTime,
    end: NaiveDateTime,
    open: Option<f64>,
    close: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    value: Option<f64>,
    volume: Option<u64>,
}

impl Candle {
    /// Построить свечу из границ времени и набора OHLCV с проверкой инвариантов.
    pub fn try_new(
        begin: NaiveDateTime,
        end: NaiveDateTime,
        ohlcv: CandleOhlcv,
    ) -> Result<Self, ParseCandleError> {
        if begin > end {
            return Err(ParseCandleError::InvalidDateRange { begin, end });
        }

        let volume = optional_non_negative(ohlcv.volume, ParseCandleError::NegativeVolume)?;

        Ok(Self {
            begin,
            end,
            open: ohlcv.open,
            close: ohlcv.close,
            high: ohlcv.high,
            low: ohlcv.low,
            value: ohlcv.value,
            volume,
        })
    }

    /// Время начала свечи.
    pub fn begin(&self) -> NaiveDateTime {
        self.begin
    }

    /// Время окончания свечи.
    pub fn end(&self) -> NaiveDateTime {
        self.end
    }

    /// Цена открытия.
    pub fn open(&self) -> Option<f64> {
        self.open
    }

    /// Цена закрытия.
    pub fn close(&self) -> Option<f64> {
        self.close
    }

    /// Максимальная цена.
    pub fn high(&self) -> Option<f64> {
        self.high
    }

    /// Минимальная цена.
    pub fn low(&self) -> Option<f64> {
        self.low
    }

    /// Объём в денежном выражении.
    pub fn value(&self) -> Option<f64> {
        self.value
    }

    /// Объём в лотах/штуках.
    pub fn volume(&self) -> Option<u64> {
        self.volume
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Компоненты OHLCV для построения [`Candle`].
pub struct CandleOhlcv {
    open: Option<f64>,
    close: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    value: Option<f64>,
    volume: Option<i64>,
}

impl CandleOhlcv {
    /// Создать набор OHLCV-значений без валидации.
    pub fn new(
        open: Option<f64>,
        close: Option<f64>,
        high: Option<f64>,
        low: Option<f64>,
        value: Option<f64>,
        volume: Option<i64>,
    ) -> Self {
        Self {
            open,
            close,
            high,
            low,
            value,
            volume,
        }
    }
}

tuple_row_conversion!(CandleBorderRow => CandleBorder, ParseCandleBorderError; 0, 1, 2, 3);

impl TryFrom<CandleRow> for Candle {
    type Error = ParseCandleError;

    fn try_from(row: CandleRow) -> Result<Self, Self::Error> {
        Self::try_new(
            row.0,
            row.1,
            CandleOhlcv::new(row.2, row.3, row.4, row.5, row.6, row.7),
        )
    }
}

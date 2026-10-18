//! Индексы MOEX и их состав.

use chrono::NaiveDate;

use super::super::errors::{ParseIndexAnalyticsError, ParseIndexError};
use super::super::helpers::non_empty_text;
use super::super::identifiers::{IndexId, SecId};
use super::super::wire::IndexAnalyticsRow;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Индекс MOEX (`indices`).
pub struct Index {
    id: IndexId,
    short_name: Box<str>,
    from: Option<NaiveDate>,
    till: Option<NaiveDate>,
}

impl Index {
    /// Построить индекс из wire-значений ISS с валидацией инвариантов.
    pub fn try_new(
        id: String,
        short_name: String,
        from: Option<NaiveDate>,
        till: Option<NaiveDate>,
    ) -> Result<Self, ParseIndexError> {
        let id = IndexId::try_from(id)?;
        let short_name = non_empty_text(short_name, ParseIndexError::EmptyShortName)?;
        if let (Some(from_date), Some(till_date)) = (from, till)
            && from_date > till_date
        {
            return Err(ParseIndexError::InvalidDateRange {
                from: from_date,
                till: till_date,
            });
        }

        Ok(Self {
            id,
            short_name,
            from,
            till,
        })
    }

    /// Идентификатор индекса (`indexid`).
    pub fn id(&self) -> &IndexId {
        &self.id
    }

    /// Краткое наименование индекса.
    pub fn short_name(&self) -> &str {
        self.short_name.as_ref()
    }

    /// Дата начала действия индекса, если задана.
    pub fn from(&self) -> Option<NaiveDate> {
        self.from
    }

    /// Дата окончания действия индекса, если задана.
    pub fn till(&self) -> Option<NaiveDate> {
        self.till
    }

    /// Проверить, что индекс активен на указанную дату.
    pub fn is_active_on(&self, date: NaiveDate) -> bool {
        self.from.is_none_or(|from| from <= date) && self.till.is_none_or(|till| date <= till)
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Компонент индекса из таблицы `analytics`.
pub struct IndexAnalytics {
    indexid: IndexId,
    tradedate: NaiveDate,
    ticker: SecId,
    shortnames: Box<str>,
    secid: SecId,
    weight: f64,
    tradingsession: u8,
    trade_session_date: NaiveDate,
}

impl TryFrom<IndexAnalyticsRow> for IndexAnalytics {
    type Error = ParseIndexAnalyticsError;

    fn try_from(row: IndexAnalyticsRow) -> Result<Self, Self::Error> {
        let IndexAnalyticsRow(
            indexid,
            tradedate,
            ticker,
            shortnames,
            secid,
            weight,
            tradingsession,
            trade_session_date,
        ) = row;

        let indexid = IndexId::try_from(indexid)?;
        let ticker = SecId::try_from(ticker).map_err(ParseIndexAnalyticsError::InvalidTicker)?;
        let secid = SecId::try_from(secid).map_err(ParseIndexAnalyticsError::InvalidSecId)?;

        let shortnames = non_empty_text(shortnames, ParseIndexAnalyticsError::EmptyShortnames)?;
        if !weight.is_finite() {
            return Err(ParseIndexAnalyticsError::NonFiniteWeight);
        }
        if weight.is_sign_negative() {
            return Err(ParseIndexAnalyticsError::NegativeWeight);
        }
        if !(1..=3).contains(&tradingsession) {
            return Err(ParseIndexAnalyticsError::InvalidTradingsession(
                tradingsession,
            ));
        }

        Ok(Self {
            indexid,
            tradedate,
            ticker,
            shortnames,
            secid,
            weight,
            tradingsession: tradingsession as u8,
            trade_session_date,
        })
    }
}

impl IndexAnalytics {
    /// Идентификатор индекса (`indexid`).
    pub fn indexid(&self) -> &IndexId {
        &self.indexid
    }

    /// Дата торгов (`tradedate`).
    pub fn tradedate(&self) -> NaiveDate {
        self.tradedate
    }

    /// Тикер компонента.
    pub fn ticker(&self) -> &SecId {
        &self.ticker
    }

    /// Краткие имена бумаг.
    pub fn shortnames(&self) -> &str {
        self.shortnames.as_ref()
    }

    /// Идентификатор инструмента компонента.
    pub fn secid(&self) -> &SecId {
        &self.secid
    }

    /// Вес компонента в индексе.
    pub fn weight(&self) -> f64 {
        self.weight
    }

    /// Торговая сессия (`1..=3`).
    pub fn tradingsession(&self) -> u8 {
        self.tradingsession
    }

    /// Дата торговой сессии (`tradedate` в источнике `analytics`).
    pub fn trade_session_date(&self) -> NaiveDate {
        self.trade_session_date
    }
}

tuple_row_conversion!(IndexRow => Index, ParseIndexError; 0, 1, 2, 3);

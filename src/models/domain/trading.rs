//! Сделки, стакан, обороты и статистика торгов.

use chrono::{NaiveDateTime, NaiveTime};

use super::super::errors::{
    ParseOrderbookError, ParseSecStatError, ParseTradeError, ParseTurnoverError,
};
use super::super::helpers::{
    into_trimmed_boxed_str, non_empty_text, optional_non_negative, positive_integer,
};
use super::super::identifiers::{BoardId, SecId};
use super::super::wire::SecStatRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Направление заявки в стакане.
pub enum BuySell {
    /// Покупка (`B`).
    Buy,
    /// Продажа (`S`).
    Sell,
}

impl BuySell {
    /// Вернуть строковый код для ISS (`B` или `S`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "B",
            Self::Sell => "S",
        }
    }
}

impl TryFrom<String> for BuySell {
    type Error = ParseOrderbookError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.trim() {
            "B" => Ok(Self::Buy),
            "S" => Ok(Self::Sell),
            _ => Err(ParseOrderbookError::InvalidSide(into_trimmed_boxed_str(
                value,
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Сделка (`trades`).
pub struct Trade {
    tradeno: u64,
    tradetime: NaiveTime,
    price: Option<f64>,
    quantity: Option<u64>,
    value: Option<f64>,
}

impl Trade {
    /// Построить сделку из wire-значений ISS с валидацией инвариантов.
    pub fn try_new(
        tradeno: i64,
        tradetime: NaiveTime,
        price: Option<f64>,
        quantity: Option<i64>,
        value: Option<f64>,
    ) -> Result<Self, ParseTradeError> {
        let tradeno = positive_integer(
            tradeno,
            ParseTradeError::NonPositiveTradeNo,
            ParseTradeError::TradeNoOutOfRange,
        )?;

        let quantity = optional_non_negative(quantity, ParseTradeError::NegativeQuantity)?;

        Ok(Self {
            tradeno,
            tradetime,
            price,
            quantity,
            value,
        })
    }

    /// Уникальный номер сделки.
    pub fn tradeno(&self) -> u64 {
        self.tradeno
    }

    /// Время сделки.
    pub fn tradetime(&self) -> NaiveTime {
        self.tradetime
    }

    /// Цена сделки.
    pub fn price(&self) -> Option<f64> {
        self.price
    }

    /// Количество в сделке.
    pub fn quantity(&self) -> Option<u64> {
        self.quantity
    }

    /// Объём сделки в денежном выражении.
    pub fn value(&self) -> Option<f64> {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Уровень стакана (`orderbook`).
pub struct OrderbookLevel {
    buy_sell: BuySell,
    price: f64,
    quantity: u64,
}

impl OrderbookLevel {
    /// Построить уровень стакана из wire-значений ISS с валидацией.
    pub fn try_new(
        buy_sell: String,
        price: Option<f64>,
        quantity: Option<i64>,
    ) -> Result<Self, ParseOrderbookError> {
        let buy_sell = BuySell::try_from(buy_sell)?;

        let Some(price) = price else {
            return Err(ParseOrderbookError::MissingPrice);
        };
        if price.is_sign_negative() {
            return Err(ParseOrderbookError::NegativePrice);
        }

        let Some(quantity) = quantity else {
            return Err(ParseOrderbookError::MissingQuantity);
        };
        let quantity = match quantity {
            raw if raw >= 0 => raw as u64,
            raw => return Err(ParseOrderbookError::NegativeQuantity(raw)),
        };

        Ok(Self {
            buy_sell,
            price,
            quantity,
        })
    }

    /// Направление заявки (`buy`/`sell`).
    pub fn buy_sell(&self) -> BuySell {
        self.buy_sell
    }

    /// Цена уровня стакана.
    pub fn price(&self) -> f64 {
        self.price
    }

    /// Количество на уровне стакана.
    pub fn quantity(&self) -> u64 {
        self.quantity
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Обороты торгов (`turnovers`).
pub struct Turnover {
    name: Box<str>,
    id: u32,
    valtoday: Option<f64>,
    valtoday_usd: Option<f64>,
    numtrades: Option<u64>,
    updatetime: NaiveDateTime,
    title: Box<str>,
}

impl Turnover {
    /// Построить запись оборотов из wire-значений ISS с валидацией инвариантов.
    pub fn try_new(
        name: String,
        id: i64,
        valtoday: Option<f64>,
        valtoday_usd: Option<f64>,
        numtrades: Option<i64>,
        updatetime: NaiveDateTime,
        title: String,
    ) -> Result<Self, ParseTurnoverError> {
        let name = non_empty_text(name, ParseTurnoverError::EmptyName)?;

        let id = positive_integer(
            id,
            ParseTurnoverError::NonPositiveId,
            ParseTurnoverError::IdOutOfRange,
        )?;

        let numtrades = optional_non_negative(numtrades, ParseTurnoverError::NegativeNumTrades)?;

        let title = non_empty_text(title, ParseTurnoverError::EmptyTitle)?;

        Ok(Self {
            name,
            id,
            valtoday,
            valtoday_usd,
            numtrades,
            updatetime,
            title,
        })
    }

    /// Наименование строки оборотов (`NAME`).
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Числовой идентификатор (`ID`).
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Оборот в рублях (`VALTODAY`).
    pub fn valtoday(&self) -> Option<f64> {
        self.valtoday
    }

    /// Оборот в долларах (`VALTODAY_USD`).
    pub fn valtoday_usd(&self) -> Option<f64> {
        self.valtoday_usd
    }

    /// Количество сделок (`NUMTRADES`).
    pub fn numtrades(&self) -> Option<u64> {
        self.numtrades
    }

    /// Время обновления (`UPDATETIME`).
    pub fn updatetime(&self) -> NaiveDateTime {
        self.updatetime
    }

    /// Человекочитаемый заголовок (`TITLE`).
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Статистика торгов по инструментам (`secstats`).
pub struct SecStat {
    secid: SecId,
    boardid: BoardId,
    voltoday: Option<u64>,
    valtoday: Option<f64>,
    highbid: Option<f64>,
    lowoffer: Option<f64>,
    lastoffer: Option<f64>,
    lastbid: Option<f64>,
    open: Option<f64>,
    low: Option<f64>,
    high: Option<f64>,
    last: Option<f64>,
    numtrades: Option<u64>,
    waprice: Option<f64>,
}

impl SecStat {
    /// Идентификатор инструмента (`SECID`).
    pub fn secid(&self) -> &SecId {
        &self.secid
    }

    /// Идентификатор режима торгов (`BOARDID`).
    pub fn boardid(&self) -> &BoardId {
        &self.boardid
    }

    /// Объём в лотах/штуках за день (`VOLTODAY`).
    pub fn voltoday(&self) -> Option<u64> {
        self.voltoday
    }

    /// Оборот в денежном выражении (`VALTODAY`).
    pub fn valtoday(&self) -> Option<f64> {
        self.valtoday
    }

    /// Лучшая цена спроса (`HIGHBID`).
    pub fn highbid(&self) -> Option<f64> {
        self.highbid
    }

    /// Лучшая цена предложения (`LOWOFFER`).
    pub fn lowoffer(&self) -> Option<f64> {
        self.lowoffer
    }

    /// Последняя цена предложения (`LASTOFFER`).
    pub fn lastoffer(&self) -> Option<f64> {
        self.lastoffer
    }

    /// Последняя цена спроса (`LASTBID`).
    pub fn lastbid(&self) -> Option<f64> {
        self.lastbid
    }

    /// Цена открытия (`OPEN`).
    pub fn open(&self) -> Option<f64> {
        self.open
    }

    /// Минимальная цена (`LOW`).
    pub fn low(&self) -> Option<f64> {
        self.low
    }

    /// Максимальная цена (`HIGH`).
    pub fn high(&self) -> Option<f64> {
        self.high
    }

    /// Последняя цена (`LAST`).
    pub fn last(&self) -> Option<f64> {
        self.last
    }

    /// Количество сделок (`NUMTRADES`).
    pub fn numtrades(&self) -> Option<u64> {
        self.numtrades
    }

    /// Средневзвешенная цена (`WAPRICE`).
    pub fn waprice(&self) -> Option<f64> {
        self.waprice
    }
}

impl TryFrom<SecStatRow> for SecStat {
    type Error = ParseSecStatError;

    fn try_from(row: SecStatRow) -> Result<Self, Self::Error> {
        let SecStatRow(
            secid,
            boardid,
            voltoday,
            valtoday,
            highbid,
            lowoffer,
            lastoffer,
            lastbid,
            open,
            low,
            high,
            last,
            numtrades,
            waprice,
        ) = row;

        let secid = SecId::try_from(secid)?;
        let boardid = BoardId::try_from(boardid)?;

        let voltoday = optional_non_negative(voltoday, ParseSecStatError::NegativeVolToday)?;

        let numtrades = optional_non_negative(numtrades, ParseSecStatError::NegativeNumTrades)?;

        Ok(Self {
            secid,
            boardid,
            voltoday,
            valtoday,
            highbid,
            lowoffer,
            lastoffer,
            lastbid,
            open,
            low,
            high,
            last,
            numtrades,
            waprice,
        })
    }
}

tuple_row_conversion!(TurnoverRow => Turnover, ParseTurnoverError; 0, 1, 2, 3, 4, 5, 6);

tuple_row_conversion!(TradeRow => Trade, ParseTradeError; 0, 1, 2, 3, 4);

tuple_row_conversion!(OrderbookLevelRow => OrderbookLevel, ParseOrderbookError; 0, 1, 2);

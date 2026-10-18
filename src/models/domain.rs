//! Доменные модели и конструкторы, сохраняющие инварианты ISS.

macro_rules! tuple_row_conversion {
    ($row:ident => $model:ty, $error:ty; $($field:tt),+ $(,)?) => {
        impl TryFrom<$crate::models::wire::$row> for $model {
            type Error = $error;

            fn try_from(row: $crate::models::wire::$row) -> Result<Self, Self::Error> {
                Self::try_new($(row.$field),+)
            }
        }
    };
}

mod candles;
mod catalog;
mod history;
mod indexes;
mod news;
mod trading;

pub use candles::{Candle, CandleBorder, CandleInterval, CandleOhlcv, CandleQuery};
pub use catalog::{Board, Engine, Market, Security, SecurityBoard, SecuritySnapshot};
pub use history::{HistoryDates, HistoryQuery, HistoryRecord};
pub use indexes::{Index, IndexAnalytics};
pub use news::{Event, SiteNews};
pub use trading::{BuySell, OrderbookLevel, SecStat, Trade, Turnover};

use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Режим получения страницы данных ISS.
///
/// Позволяет единообразно описать: первую страницу, произвольную страницу
/// (`start`, `limit`) или полную выборку с авто-пагинацией.
pub enum PageRequest {
    /// Первая страница ISS (без явных `start`, `limit`).
    #[default]
    FirstPage,
    /// Явные параметры пагинации ISS.
    Page(Pagination),
    /// Полная выгрузка с авто-пагинацией и размером страницы.
    All {
        /// Размер страницы ISS (`limit`) при авто-пагинации.
        page_limit: NonZeroU32,
    },
}

impl PageRequest {
    /// Запросить первую страницу ISS.
    pub fn first_page() -> Self {
        Self::FirstPage
    }

    /// Запросить страницу ISS с явными параметрами.
    pub fn page(pagination: Pagination) -> Self {
        Self::Page(pagination)
    }

    /// Запросить полную выборку ISS с авто-пагинацией.
    pub fn all(page_limit: NonZeroU32) -> Self {
        Self::All { page_limit }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
/// Параметры пагинации ISS API (`start`, `limit`).
#[must_use]
pub struct Pagination {
    /// Смещение первой записи (`start`).
    pub start: Option<u32>,
    /// Максимальный размер страницы (`limit`).
    pub limit: Option<NonZeroU32>,
}

impl Pagination {
    /// Вернуть копию с установленным `start`.
    pub fn with_start(mut self, start: u32) -> Self {
        self.start = Some(start);
        self
    }

    /// Вернуть копию с установленным `limit`.
    pub fn with_limit(mut self, limit: NonZeroU32) -> Self {
        self.limit = Some(limit);
        self
    }
}

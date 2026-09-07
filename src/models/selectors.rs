//! Утилиты fluent-интерфейса для отбора и сортировки доменных коллекций.

use std::borrow::Borrow;

use super::{Index, IndexAnalytics, SecurityBoard};

/// Методы fluent-интерфейса для коллекции индексов.
pub trait IndexesExt {
    /// Оставить только индексы с максимальным `till`.
    fn retain_actual_by_till(&mut self);

    /// Вернуть коллекцию индексов только с максимальным `till`.
    #[must_use]
    fn into_actual_by_till(mut self) -> Self
    where
        Self: Sized,
    {
        self.retain_actual_by_till();
        self
    }
}

impl IndexesExt for Vec<Index> {
    fn retain_actual_by_till(&mut self) {
        let latest_till = self.iter().filter_map(Index::till).max();
        self.retain(|index| index.till() == latest_till);
    }
}

/// Методы fluent-интерфейса для коллекции состава индекса.
pub trait IndexAnalyticsExt {
    /// Оставить только актуальную торговую сессию:
    /// максимальные `trade_session_date` и `tradingsession`.
    fn retain_actual_by_session(&mut self);

    /// Вернуть только актуальную торговую сессию.
    #[must_use]
    fn into_actual_by_session(mut self) -> Self
    where
        Self: Sized,
    {
        self.retain_actual_by_session();
        self
    }

    /// Отсортировать по убыванию `weight` и затем по `secid`.
    fn sort_by_weight_desc(&mut self);

    /// Вернуть отсортированную по убыванию `weight` коллекцию.
    #[must_use]
    fn into_sorted_by_weight_desc(mut self) -> Self
    where
        Self: Sized,
    {
        self.sort_by_weight_desc();
        self
    }
}

impl IndexAnalyticsExt for Vec<IndexAnalytics> {
    fn retain_actual_by_session(&mut self) {
        let session_key =
            |item: &IndexAnalytics| (item.trade_session_date(), item.tradingsession());
        if let Some(latest_session) = self.iter().map(session_key).max() {
            self.retain(|item| session_key(item) == latest_session);
        }
    }

    fn sort_by_weight_desc(&mut self) {
        self.sort_by(|left, right| {
            right
                .weight()
                .total_cmp(&left.weight())
                .then_with(|| left.secid().as_str().cmp(right.secid().as_str()))
        });
    }
}

/// Методы fluent-интерфейса для выбора режима торгов из коллекции `boards`.
pub trait SecurityBoardsExt {
    /// Найти первичный `stock`-режим (`is_primary=1`) или первый `stock`-режим.
    fn stock_primary_or_first(&self) -> Option<&SecurityBoard>;

    /// Вернуть первичный `stock`-режим (`is_primary=1`) или первый `stock`-режим.
    fn into_stock_primary_or_first(self) -> Option<SecurityBoard>
    where
        Self: Sized;
}

impl SecurityBoardsExt for Vec<SecurityBoard> {
    fn stock_primary_or_first(&self) -> Option<&SecurityBoard> {
        stock_primary_or_first(self.iter())
    }

    fn into_stock_primary_or_first(self) -> Option<SecurityBoard> {
        stock_primary_or_first(self)
    }
}

fn stock_primary_or_first<T: Borrow<SecurityBoard>>(
    boards: impl IntoIterator<Item = T>,
) -> Option<T> {
    let mut fallback = None;
    for board in boards {
        let value = board.borrow();
        if value.engine().as_str() != "stock" {
            continue;
        }
        if value.is_primary() {
            return Some(board);
        }
        if fallback.is_none() {
            fallback = Some(board);
        }
    }
    fallback
}

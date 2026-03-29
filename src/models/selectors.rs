//! Утилиты fluent-интерфейса для отбора и сортировки доменных коллекций.

use std::cmp::Ordering;

use super::{Index, IndexAnalytics, SecurityBoard};

/// Методы fluent-интерфейса для коллекции индексов.
pub trait IndexesExt {
    /// Оставить только индексы с максимальным `till`.
    fn retain_actual_by_till(&mut self);

    /// Вернуть коллекцию индексов только с максимальным `till`.
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
        // Сначала отбираем записи за последнюю дату торговой сессии.
        let Some(latest_trade_session_date) =
            self.iter().map(IndexAnalytics::trade_session_date).max()
        else {
            return;
        };
        self.retain(|item| item.trade_session_date() == latest_trade_session_date);

        // После фильтра по дате оставляем максимальный номер сессии.
        let Some(latest_tradingsession) = self.iter().map(IndexAnalytics::tradingsession).max()
        else {
            return;
        };
        self.retain(|item| item.tradingsession() == latest_tradingsession);
    }

    fn sort_by_weight_desc(&mut self) {
        self.sort_by(|left, right| {
            right
                .weight()
                // `weight` заранее валидируется как конечное число, но fallback
                // защищает от нарушения инварианта в будущем.
                .partial_cmp(&left.weight())
                .unwrap_or(Ordering::Equal)
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
        let mut fallback = None;
        for board in self {
            if board.engine().as_str() != "stock" {
                continue;
            }
            if board.is_primary() {
                return Some(board);
            }
            if fallback.is_none() {
                fallback = Some(board);
            }
        }
        fallback
    }

    fn into_stock_primary_or_first(self) -> Option<SecurityBoard> {
        let mut fallback = None;
        for board in self {
            if board.engine().as_str() != "stock" {
                continue;
            }
            if board.is_primary() {
                return Some(board);
            }
            if fallback.is_none() {
                fallback = Some(board);
            }
        }
        fallback
    }
}

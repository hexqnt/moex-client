//! Wire-структуры составных ответов ISS и внутренних строк.

use serde::Deserialize;

use crate::models::{ParseSecurityBoardError, SecurityBoard};

#[derive(Debug, Deserialize)]
/// Универсальное представление табличного блока ISS (`...: { data: [...] }`).
pub(super) struct IssTable<T> {
    pub(super) data: Vec<T>,
}

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `boards` внутри `securities/{secid}`.
pub(super) struct SecurityBoardRow(
    pub(super) String,
    pub(super) String,
    pub(super) String,
    pub(super) i64,
);

impl TryFrom<SecurityBoardRow> for SecurityBoard {
    type Error = ParseSecurityBoardError;

    fn try_from(
        SecurityBoardRow(engine, market, boardid, is_primary): SecurityBoardRow,
    ) -> Result<Self, Self::Error> {
        Self::try_new(engine, market, boardid, is_primary)
    }
}

#[derive(Debug, Deserialize)]
/// Ответ объединённого endpoint `securities + marketdata`.
pub(super) struct BoardSecuritySnapshotsResponse {
    pub(super) securities: IssTable<BoardSecurityRow>,
    pub(super) marketdata: IssTable<BoardMarketDataRow>,
}

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `securities` для режима snapshot.
pub(super) struct BoardSecurityRow(pub(super) String, pub(super) Option<i64>);

#[derive(Debug, Deserialize)]
/// Wire-строка таблицы `marketdata` для режима snapshot.
pub(super) struct BoardMarketDataRow(pub(super) String, pub(super) Option<f64>);

//! Каталог движков, рынков, режимов торгов и инструментов MOEX.

use super::super::errors::{
    ParseBoardError, ParseEngineError, ParseMarketError, ParseSecurityBoardError,
    ParseSecurityError, ParseSecuritySnapshotError,
};
use super::super::helpers::{non_empty_text, non_negative_u32, positive_integer, wire_bool};
use super::super::identifiers::{BoardId, EngineId, EngineName, MarketId, MarketName, SecId};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Торговый движок MOEX (`engines`).
pub struct Engine {
    id: EngineId,
    name: EngineName,
    title: Box<str>,
}

impl Engine {
    /// Построить движок из wire-значений ISS с валидацией инвариантов.
    pub fn try_new(id: i64, name: String, title: String) -> Result<Self, ParseEngineError> {
        let id = positive_integer(
            id,
            ParseEngineError::NonPositiveId,
            ParseEngineError::IdOutOfRange,
        )
        .map(EngineId)?;

        let name = EngineName::try_from(name)?;

        let title = non_empty_text(title, ParseEngineError::EmptyTitle)?;

        Ok(Self { id, name, title })
    }

    /// Идентификатор движка.
    pub fn id(&self) -> EngineId {
        self.id
    }

    /// Короткое имя движка, используемое в URL ISS.
    pub fn name(&self) -> &EngineName {
        &self.name
    }

    /// Человекочитаемое название движка.
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Рынок MOEX (`markets`).
pub struct Market {
    id: MarketId,
    name: MarketName,
    title: Box<str>,
}

impl Market {
    /// Построить рынок из wire-значений ISS с валидацией инвариантов.
    pub fn try_new(id: i64, name: String, title: String) -> Result<Self, ParseMarketError> {
        let id = positive_integer(
            id,
            ParseMarketError::NonPositiveId,
            ParseMarketError::IdOutOfRange,
        )
        .map(MarketId)?;

        let name = MarketName::try_from(name)?;

        let title = non_empty_text(title, ParseMarketError::EmptyTitle)?;

        Ok(Self { id, name, title })
    }

    /// Идентификатор рынка.
    pub fn id(&self) -> MarketId {
        self.id
    }

    /// Короткое имя рынка, используемое в URL ISS.
    pub fn name(&self) -> &MarketName {
        &self.name
    }

    /// Человекочитаемое название рынка.
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Режим торгов MOEX (`boards`).
pub struct Board {
    id: u32,
    board_group_id: u32,
    boardid: BoardId,
    title: Box<str>,
    is_traded: bool,
}

impl Board {
    /// Построить режим торгов из wire-значений ISS с валидацией инвариантов.
    pub fn try_new(
        id: i64,
        board_group_id: i64,
        boardid: String,
        title: String,
        is_traded: i64,
    ) -> Result<Self, ParseBoardError> {
        let id = positive_integer(
            id,
            ParseBoardError::NonPositiveId,
            ParseBoardError::IdOutOfRange,
        )?;

        let board_group_id = non_negative_u32(
            board_group_id,
            ParseBoardError::NegativeBoardGroupId,
            ParseBoardError::BoardGroupIdOutOfRange,
        )?;

        let boardid = BoardId::try_from(boardid)?;

        let title = non_empty_text(title, ParseBoardError::EmptyTitle)?;

        let is_traded = wire_bool(is_traded, ParseBoardError::InvalidIsTraded)?;

        Ok(Self {
            id,
            board_group_id,
            boardid,
            title,
            is_traded,
        })
    }

    /// Числовой идентификатор режима торгов.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Идентификатор группы board.
    pub fn board_group_id(&self) -> u32 {
        self.board_group_id
    }

    /// Символьный идентификатор режима торгов (`boardid`).
    pub fn boardid(&self) -> &BoardId {
        &self.boardid
    }

    /// Человекочитаемое название режима торгов.
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Признак, что режим предназначен для торгов (`1` в ISS).
    pub fn is_traded(&self) -> bool {
        self.is_traded
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Режим торгов инструмента из `securities/{secid}` таблицы `boards`.
pub struct SecurityBoard {
    engine: EngineName,
    market: MarketName,
    boardid: BoardId,
    is_primary: bool,
}

impl SecurityBoard {
    /// Построить режим торгов инструмента из wire-значений ISS.
    pub fn try_new(
        engine: String,
        market: String,
        boardid: String,
        is_primary: i64,
    ) -> Result<Self, ParseSecurityBoardError> {
        let engine = EngineName::try_from(engine)?;
        let market = MarketName::try_from(market)?;
        let boardid = BoardId::try_from(boardid)?;
        let is_primary = wire_bool(is_primary, ParseSecurityBoardError::InvalidIsPrimary)?;

        Ok(Self {
            engine,
            market,
            boardid,
            is_primary,
        })
    }

    /// Имя движка из ответа ISS (`engine`).
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Имя рынка из ответа ISS (`market`).
    pub fn market(&self) -> &MarketName {
        &self.market
    }

    /// Идентификатор режима торгов из ответа ISS (`boardid`).
    pub fn boardid(&self) -> &BoardId {
        &self.boardid
    }

    /// Признак первичного режима (`is_primary`).
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Инструмент MOEX (`securities`).
pub struct Security {
    secid: SecId,
    shortname: Box<str>,
    secname: Box<str>,
    status: Box<str>,
}

impl Security {
    /// Построить инструмент из wire-значений ISS с валидацией инвариантов.
    pub fn try_new(
        secid: String,
        shortname: String,
        secname: String,
        status: String,
    ) -> Result<Self, ParseSecurityError> {
        let secid = SecId::try_from(secid)?;

        let shortname = non_empty_text(shortname, ParseSecurityError::EmptyShortname)?;

        let secname = non_empty_text(secname, ParseSecurityError::EmptySecname)?;

        let status = non_empty_text(status, ParseSecurityError::EmptyStatus)?;

        Ok(Self {
            secid,
            shortname,
            secname,
            status,
        })
    }

    /// Идентификатор инструмента (`secid`).
    pub fn secid(&self) -> &SecId {
        &self.secid
    }

    /// Краткое имя инструмента.
    pub fn shortname(&self) -> &str {
        self.shortname.as_ref()
    }

    /// Полное имя инструмента.
    pub fn secname(&self) -> &str {
        self.secname.as_ref()
    }

    /// Текущий статус инструмента в ISS.
    pub fn status(&self) -> &str {
        self.status.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Снимок инструмента с полями `LOTSIZE` и `LAST`.
pub struct SecuritySnapshot {
    secid: SecId,
    lot_size: Option<u32>,
    last: Option<f64>,
}

impl SecuritySnapshot {
    /// Построить снимок инструмента из wire-значений ISS.
    pub fn try_new(
        secid: String,
        lot_size: Option<i64>,
        last: Option<f64>,
    ) -> Result<Self, ParseSecuritySnapshotError> {
        let secid = SecId::try_from(secid).map_err(ParseSecuritySnapshotError::InvalidSecId)?;
        let lot_size = Self::parse_lot_size(lot_size)?;
        Self::try_from_parts(secid, lot_size, last)
    }

    pub(crate) fn parse_lot_size(
        lot_size: Option<i64>,
    ) -> Result<Option<u32>, ParseSecuritySnapshotError> {
        lot_size
            .map(|raw| {
                non_negative_u32(
                    raw,
                    ParseSecuritySnapshotError::NegativeLotSize,
                    ParseSecuritySnapshotError::LotSizeOutOfRange,
                )
            })
            .transpose()
    }

    /// Внутренний конструктор для уже нормализованных значений snapshot-а.
    pub(crate) fn try_from_parts(
        secid: SecId,
        lot_size: Option<u32>,
        last: Option<f64>,
    ) -> Result<Self, ParseSecuritySnapshotError> {
        // Для `LAST` запрещаем NaN/Infinity, чтобы downstream-код работал с корректным числом.
        if let Some(last) = last
            && !last.is_finite()
        {
            return Err(ParseSecuritySnapshotError::NonFiniteLast(last));
        }

        Ok(Self {
            secid,
            lot_size,
            last,
        })
    }

    /// Идентификатор инструмента (`secid`).
    pub fn secid(&self) -> &SecId {
        &self.secid
    }

    /// Размер лота (`LOTSIZE`), если поле присутствует в ISS.
    pub fn lot_size(&self) -> Option<u32> {
        self.lot_size
    }

    /// Последняя цена (`LAST`), если поле присутствует в ISS.
    pub fn last(&self) -> Option<f64> {
        self.last
    }
}

tuple_row_conversion!(EngineRow => Engine, ParseEngineError; 0, 1, 2);

tuple_row_conversion!(MarketRow => Market, ParseMarketError; 0, 1, 2);

tuple_row_conversion!(BoardRow => Board, ParseBoardError; 0, 1, 2, 3, 4);

tuple_row_conversion!(SecurityRow => Security, ParseSecurityError; 0, 1, 2, 3);

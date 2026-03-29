use std::convert::Infallible;
use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Идентификатор индекса MOEX (`indexid`).
pub struct IndexId(Box<str>);

impl IndexId {
    /// Вернуть строковое представление идентификатора.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for IndexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for IndexId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&IndexId> for IndexId {
    fn from(value: &IndexId) -> Self {
        value.clone()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Index`].
pub enum ParseIndexError {
    /// Пустой `indexid`.
    #[error("index id must not be empty")]
    EmptyIndexId,
    /// Пустое краткое имя индекса.
    #[error("index short name must not be empty")]
    EmptyShortName,
    /// Границы активности индекса заданы в неверном порядке.
    #[error("invalid index date range: from={from} is after till={till}")]
    InvalidDateRange {
        /// Начальная дата периода.
        from: NaiveDate,
        /// Конечная дата периода.
        till: NaiveDate,
    },
}

impl From<Infallible> for ParseIndexError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`HistoryDates`].
pub enum ParseHistoryDatesError {
    /// Границы доступной истории заданы в неверном порядке.
    #[error("invalid history dates range: from={from} is after till={till}")]
    InvalidDateRange {
        /// Начальная дата доступной истории.
        from: NaiveDate,
        /// Конечная дата доступной истории.
        till: NaiveDate,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`HistoryRecord`].
pub enum ParseHistoryRecordError {
    /// Некорректный `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Количество сделок отрицательное.
    #[error("history numtrades must not be negative, got {0}")]
    NegativeNumTrades(i64),
    /// Объём торгов отрицательный.
    #[error("history volume must not be negative, got {0}")]
    NegativeVolume(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Turnover`].
pub enum ParseTurnoverError {
    /// Пустое поле `name`.
    #[error("turnover name must not be empty")]
    EmptyName,
    /// Идентификатор должен быть положительным.
    #[error("turnover id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор не помещается в `u32`.
    #[error("turnover id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// Количество сделок отрицательное.
    #[error("turnover numtrades must not be negative, got {0}")]
    NegativeNumTrades(i64),
    /// Пустое поле `title`.
    #[error("turnover title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`SecStat`].
pub enum ParseSecStatError {
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Некорректный `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Объём торгов отрицательный.
    #[error("secstats voltoday must not be negative, got {0}")]
    NegativeVolToday(i64),
    /// Количество сделок отрицательное.
    #[error("secstats numtrades must not be negative, got {0}")]
    NegativeNumTrades(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`SiteNews`].
pub enum ParseSiteNewsError {
    /// Идентификатор новости должен быть положительным.
    #[error("sitenews id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор новости не помещается в `u64`.
    #[error("sitenews id is out of range for u64, got {0}")]
    IdOutOfRange(i64),
    /// Пустое поле `tag`.
    #[error("sitenews tag must not be empty")]
    EmptyTag,
    /// Пустое поле `title`.
    #[error("sitenews title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Event`].
pub enum ParseEventError {
    /// Идентификатор события должен быть положительным.
    #[error("events id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор события не помещается в `u64`.
    #[error("events id is out of range for u64, got {0}")]
    IdOutOfRange(i64),
    /// Пустое поле `tag`.
    #[error("events tag must not be empty")]
    EmptyTag,
    /// Пустое поле `title`.
    #[error("events title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`IndexAnalytics`].
pub enum ParseIndexAnalyticsError {
    /// Некорректные данные самого индекса.
    #[error(transparent)]
    InvalidIndexId(#[from] ParseIndexError),
    /// Некорректный `ticker`.
    #[error("ticker is invalid: {0}")]
    InvalidTicker(ParseSecIdError),
    /// Некорректный `secid`.
    #[error("secid is invalid: {0}")]
    InvalidSecId(ParseSecIdError),
    /// Пустое поле `shortnames`.
    #[error("shortnames must not be empty")]
    EmptyShortnames,
    /// Вес компонента не является конечным числом.
    #[error("weight must be finite")]
    NonFiniteWeight,
    /// Вес компонента отрицательный.
    #[error("weight must not be negative")]
    NegativeWeight,
    /// Недопустимое значение `tradingsession`.
    #[error("tradingsession must be 1, 2 or 3, got {0}")]
    InvalidTradingsession(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Engine`].
pub enum ParseEngineError {
    /// Идентификатор движка должен быть положительным.
    #[error("engine id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор движка не помещается в `u32`.
    #[error("engine id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// Некорректное имя движка.
    #[error(transparent)]
    InvalidName(#[from] ParseEngineNameError),
    /// Пустой заголовок движка.
    #[error("engine title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора имени торгового движка.
pub enum ParseEngineNameError {
    /// Пустое имя.
    #[error("engine name must not be empty")]
    Empty,
    /// Имя содержит символ `/`, запрещённый в path-сегменте.
    #[error("engine name must not contain '/'")]
    ContainsSlash,
}

impl From<Infallible> for ParseEngineNameError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Имя торгового движка MOEX (`engine`).
pub struct EngineName(Box<str>);

impl EngineName {
    /// Вернуть строковое представление имени движка.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for EngineName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for EngineName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&EngineName> for EngineName {
    fn from(value: &EngineName) -> Self {
        value.clone()
    }
}

impl TryFrom<String> for EngineName {
    type Error = ParseEngineNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for EngineName {
    type Error = ParseEngineNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(ParseEngineNameError::Empty);
        }
        if value.contains('/') {
            return Err(ParseEngineNameError::ContainsSlash);
        }
        Ok(Self(value.to_owned().into_boxed_str()))
    }
}

impl FromStr for EngineName {
    type Err = ParseEngineNameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Market`].
pub enum ParseMarketError {
    /// Идентификатор рынка должен быть положительным.
    #[error("market id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор рынка не помещается в `u32`.
    #[error("market id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// Некорректное имя рынка.
    #[error(transparent)]
    InvalidName(#[from] ParseMarketNameError),
    /// Пустой заголовок рынка.
    #[error("market title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Board`].
pub enum ParseBoardError {
    /// Идентификатор board должен быть положительным.
    #[error("board id must be positive, got {0}")]
    NonPositiveId(i64),
    /// Идентификатор board не помещается в `u32`.
    #[error("board id is out of range for u32, got {0}")]
    IdOutOfRange(i64),
    /// `board_group_id` отрицательный.
    #[error("board_group_id must not be negative, got {0}")]
    NegativeBoardGroupId(i64),
    /// `board_group_id` не помещается в `u32`.
    #[error("board_group_id is out of range for u32, got {0}")]
    BoardGroupIdOutOfRange(i64),
    /// Некорректный текстовый `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Пустой заголовок board.
    #[error("board title must not be empty")]
    EmptyTitle,
    /// Некорректный флаг `is_traded` (допустимы только 0/1).
    #[error("is_traded must be 0 or 1, got {0}")]
    InvalidIsTraded(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Security`].
pub enum ParseSecurityError {
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Пустое поле `shortname`.
    #[error("security shortname must not be empty")]
    EmptyShortname,
    /// Пустое поле `secname`.
    #[error("security secname must not be empty")]
    EmptySecname,
    /// Пустое поле `status`.
    #[error("security status must not be empty")]
    EmptyStatus,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`SecurityBoard`].
pub enum ParseSecurityBoardError {
    /// Некорректное имя движка.
    #[error(transparent)]
    InvalidEngine(#[from] ParseEngineNameError),
    /// Некорректное имя рынка.
    #[error(transparent)]
    InvalidMarket(#[from] ParseMarketNameError),
    /// Некорректный `boardid`.
    #[error(transparent)]
    InvalidBoardId(#[from] ParseBoardIdError),
    /// Некорректный флаг `is_primary` (допустимы только 0/1).
    #[error("is_primary must be 0 or 1, got {0}")]
    InvalidIsPrimary(i64),
}

#[derive(Debug, Error, Clone, PartialEq)]
/// Ошибки построения [`SecuritySnapshot`].
pub enum ParseSecuritySnapshotError {
    /// Некорректный `secid`.
    #[error(transparent)]
    InvalidSecId(#[from] ParseSecIdError),
    /// Размер лота отрицательный.
    #[error("lot size must not be negative, got {0}")]
    NegativeLotSize(i64),
    /// Размер лота не помещается в `u32`.
    #[error("lot size is out of range for u32, got {0}")]
    LotSizeOutOfRange(i64),
    /// Значение `last` не является конечным числом.
    #[error("last must be finite")]
    NonFiniteLast(f64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Candle`].
pub enum ParseCandleError {
    /// Границы свечи заданы в неверном порядке.
    #[error("invalid candle datetime range: begin={begin} is after end={end}")]
    InvalidDateRange {
        /// Начало свечи.
        begin: NaiveDateTime,
        /// Конец свечи.
        end: NaiveDateTime,
    },
    /// Объём свечи отрицательный.
    #[error("candle volume must not be negative, got {0}")]
    NegativeVolume(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора значения интервала свечей.
pub enum ParseCandleIntervalError {
    /// Неизвестный код интервала ISS.
    #[error("invalid candle interval code, got {0}")]
    InvalidCode(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`CandleQuery`].
pub enum ParseCandleQueryError {
    /// В запросе свечей `from` больше `till`.
    #[error("invalid candle query datetime range: from={from} is after till={till}")]
    InvalidDateRange {
        /// Начальная дата и время выборки.
        from: NaiveDateTime,
        /// Конечная дата и время выборки.
        till: NaiveDateTime,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`CandleBorder`].
pub enum ParseCandleBorderError {
    /// Границы диапазона заданы в неверном порядке.
    #[error("invalid candle borders range: begin={begin} is after end={end}")]
    InvalidDateRange {
        /// Начало доступного диапазона.
        begin: NaiveDateTime,
        /// Конец доступного диапазона.
        end: NaiveDateTime,
    },
    /// Некорректный код интервала.
    #[error(transparent)]
    InvalidInterval(#[from] ParseCandleIntervalError),
    /// `board_group_id` отрицательный.
    #[error("board_group_id must not be negative, got {0}")]
    NegativeBoardGroupId(i64),
    /// `board_group_id` не помещается в `u32`.
    #[error("board_group_id is out of range for u32, got {0}")]
    BoardGroupIdOutOfRange(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`Trade`].
pub enum ParseTradeError {
    /// Номер сделки должен быть положительным.
    #[error("trade number must be positive, got {0}")]
    NonPositiveTradeNo(i64),
    /// Номер сделки не помещается в `u64`.
    #[error("trade number is out of range for u64, got {0}")]
    TradeNoOutOfRange(i64),
    /// Количество в сделке отрицательное.
    #[error("trade quantity must not be negative, got {0}")]
    NegativeQuantity(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки построения [`OrderbookLevel`].
pub enum ParseOrderbookError {
    /// Некорректное направление заявки (`B`/`S`).
    #[error("orderbook side must be 'B' or 'S', got '{0}'")]
    InvalidSide(Box<str>),
    /// Отсутствует цена уровня стакана.
    #[error("orderbook price must be present")]
    MissingPrice,
    /// Цена уровня стакана отрицательная.
    #[error("orderbook price must not be negative")]
    NegativePrice,
    /// Отсутствует объём уровня стакана.
    #[error("orderbook quantity must be present")]
    MissingQuantity,
    /// Объём уровня стакана отрицательный.
    #[error("orderbook quantity must not be negative, got {0}")]
    NegativeQuantity(i64),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора идентификатора инструмента (`secid`).
pub enum ParseSecIdError {
    /// Пустой `secid`.
    #[error("secid must not be empty")]
    Empty,
    /// `secid` содержит символ `/`, запрещённый в path-сегменте.
    #[error("secid must not contain '/'")]
    ContainsSlash,
}

impl From<Infallible> for ParseSecIdError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Идентификатор инструмента MOEX (`secid`).
pub struct SecId(Box<str>);

impl SecId {
    /// Вернуть строковое представление идентификатора.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for SecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for SecId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&SecId> for SecId {
    fn from(value: &SecId) -> Self {
        value.clone()
    }
}

impl TryFrom<String> for SecId {
    type Error = ParseSecIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for SecId {
    type Error = ParseSecIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(ParseSecIdError::Empty);
        }
        if value.contains('/') {
            return Err(ParseSecIdError::ContainsSlash);
        }
        Ok(Self(value.to_owned().into_boxed_str()))
    }
}

impl FromStr for SecId {
    type Err = ParseSecIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора идентификатора режима торгов (`boardid`).
pub enum ParseBoardIdError {
    /// Пустой `boardid`.
    #[error("boardid must not be empty")]
    Empty,
    /// `boardid` содержит символ `/`, запрещённый в path-сегменте.
    #[error("boardid must not contain '/'")]
    ContainsSlash,
}

impl From<Infallible> for ParseBoardIdError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Идентификатор режима торгов MOEX (`boardid`).
pub struct BoardId(Box<str>);

impl BoardId {
    /// Вернуть строковое представление идентификатора.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for BoardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for BoardId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&BoardId> for BoardId {
    fn from(value: &BoardId) -> Self {
        value.clone()
    }
}

impl TryFrom<String> for BoardId {
    type Error = ParseBoardIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for BoardId {
    type Error = ParseBoardIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(ParseBoardIdError::Empty);
        }
        if value.contains('/') {
            return Err(ParseBoardIdError::ContainsSlash);
        }
        Ok(Self(value.to_owned().into_boxed_str()))
    }
}

impl FromStr for BoardId {
    type Err = ParseBoardIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
/// Ошибки разбора имени рынка MOEX.
pub enum ParseMarketNameError {
    /// Пустое имя.
    #[error("market name must not be empty")]
    Empty,
    /// Имя содержит символ `/`, запрещённый в path-сегменте.
    #[error("market name must not contain '/'")]
    ContainsSlash,
}

impl From<Infallible> for ParseMarketNameError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Имя рынка MOEX (`market`).
pub struct MarketName(Box<str>);

impl MarketName {
    /// Вернуть строковое представление имени рынка.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for MarketName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for MarketName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&MarketName> for MarketName {
    fn from(value: &MarketName) -> Self {
        value.clone()
    }
}

impl TryFrom<String> for MarketName {
    type Error = ParseMarketNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for MarketName {
    type Error = ParseMarketNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(ParseMarketNameError::Empty);
        }
        if value.contains('/') {
            return Err(ParseMarketNameError::ContainsSlash);
        }
        Ok(Self(value.to_owned().into_boxed_str()))
    }
}

impl FromStr for MarketName {
    type Err = ParseMarketNameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Идентификатор торгового движка MOEX.
pub struct EngineId(u32);

impl EngineId {
    /// Вернуть числовое значение идентификатора.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for EngineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
        if id <= 0 {
            return Err(ParseEngineError::NonPositiveId(id));
        }
        let id = u32::try_from(id)
            .map(EngineId)
            .map_err(|_| ParseEngineError::IdOutOfRange(id))?;

        let name = EngineName::try_from(name)?;

        let title = title.trim();
        if title.is_empty() {
            return Err(ParseEngineError::EmptyTitle);
        }

        Ok(Self {
            id,
            name,
            title: title.to_owned().into_boxed_str(),
        })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Идентификатор рынка MOEX.
pub struct MarketId(u32);

impl MarketId {
    /// Вернуть числовое значение идентификатора.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for MarketId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
        if id <= 0 {
            return Err(ParseMarketError::NonPositiveId(id));
        }
        let id = u32::try_from(id)
            .map(MarketId)
            .map_err(|_| ParseMarketError::IdOutOfRange(id))?;

        let name = MarketName::try_from(name)?;

        let title = title.trim();
        if title.is_empty() {
            return Err(ParseMarketError::EmptyTitle);
        }

        Ok(Self {
            id,
            name,
            title: title.to_owned().into_boxed_str(),
        })
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
        if id <= 0 {
            return Err(ParseBoardError::NonPositiveId(id));
        }
        let id = u32::try_from(id).map_err(|_| ParseBoardError::IdOutOfRange(id))?;

        if board_group_id < 0 {
            return Err(ParseBoardError::NegativeBoardGroupId(board_group_id));
        }
        let board_group_id = u32::try_from(board_group_id)
            .map_err(|_| ParseBoardError::BoardGroupIdOutOfRange(board_group_id))?;

        let boardid = BoardId::try_from(boardid)?;

        let title = title.trim();
        if title.is_empty() {
            return Err(ParseBoardError::EmptyTitle);
        }

        let is_traded = match is_traded {
            0 => false,
            1 => true,
            other => return Err(ParseBoardError::InvalidIsTraded(other)),
        };

        Ok(Self {
            id,
            board_group_id,
            boardid,
            title: title.to_owned().into_boxed_str(),
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
        let is_primary = match is_primary {
            0 => false,
            1 => true,
            other => return Err(ParseSecurityBoardError::InvalidIsPrimary(other)),
        };

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

        let shortname = shortname.trim();
        if shortname.is_empty() {
            return Err(ParseSecurityError::EmptyShortname);
        }

        let secname = secname.trim();
        if secname.is_empty() {
            return Err(ParseSecurityError::EmptySecname);
        }

        let status = status.trim();
        if status.is_empty() {
            return Err(ParseSecurityError::EmptyStatus);
        }

        Ok(Self {
            secid,
            shortname: shortname.to_owned().into_boxed_str(),
            secname: secname.to_owned().into_boxed_str(),
            status: status.to_owned().into_boxed_str(),
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
        let lot_size = match lot_size {
            None => None,
            Some(raw) if raw < 0 => return Err(ParseSecuritySnapshotError::NegativeLotSize(raw)),
            Some(raw) => Some(
                u32::try_from(raw)
                    .map_err(|_| ParseSecuritySnapshotError::LotSizeOutOfRange(raw))?,
            ),
        };
        Self::try_from_parts(secid, lot_size, last)
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
        if board_group_id < 0 {
            return Err(ParseCandleBorderError::NegativeBoardGroupId(board_group_id));
        }
        let board_group_id = u32::try_from(board_group_id)
            .map_err(|_| ParseCandleBorderError::BoardGroupIdOutOfRange(board_group_id))?;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
/// Параметры пагинации ISS API (`start`, `limit`).
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

        let volume = match ohlcv.volume {
            None => None,
            Some(raw) if raw >= 0 => Some(raw as u64),
            Some(raw) => return Err(ParseCandleError::NegativeVolume(raw)),
        };

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
        if tradeno <= 0 {
            return Err(ParseTradeError::NonPositiveTradeNo(tradeno));
        }
        let tradeno =
            u64::try_from(tradeno).map_err(|_| ParseTradeError::TradeNoOutOfRange(tradeno))?;

        let quantity = match quantity {
            None => None,
            Some(raw) if raw >= 0 => Some(raw as u64),
            Some(raw) => return Err(ParseTradeError::NegativeQuantity(raw)),
        };

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
        let value = value.trim();
        match value {
            "B" => Ok(Self::Buy),
            "S" => Ok(Self::Sell),
            _ => Err(ParseOrderbookError::InvalidSide(
                value.to_owned().into_boxed_str(),
            )),
        }
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

impl TryFrom<String> for IndexId {
    type Error = ParseIndexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for IndexId {
    type Error = ParseIndexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(ParseIndexError::EmptyIndexId);
        }
        Ok(Self(value.to_owned().into_boxed_str()))
    }
}

impl FromStr for IndexId {
    type Err = ParseIndexError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Индекс MOEX (`indices`).
pub struct Index {
    id: IndexId,
    short_name: Box<str>,
    from: Option<NaiveDate>,
    till: Option<NaiveDate>,
}

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
        let name = name.trim();
        if name.is_empty() {
            return Err(ParseTurnoverError::EmptyName);
        }

        if id <= 0 {
            return Err(ParseTurnoverError::NonPositiveId(id));
        }
        let id = u32::try_from(id).map_err(|_| ParseTurnoverError::IdOutOfRange(id))?;

        let numtrades = match numtrades {
            None => None,
            Some(raw) if raw >= 0 => Some(raw as u64),
            Some(raw) => return Err(ParseTurnoverError::NegativeNumTrades(raw)),
        };

        let title = title.trim();
        if title.is_empty() {
            return Err(ParseTurnoverError::EmptyTitle);
        }

        Ok(Self {
            name: name.to_owned().into_boxed_str(),
            id,
            valtoday,
            valtoday_usd,
            numtrades,
            updatetime,
            title: title.to_owned().into_boxed_str(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Новость MOEX ISS (`sitenews`).
pub struct SiteNews {
    id: u64,
    tag: Box<str>,
    title: Box<str>,
    published_at: NaiveDateTime,
    modified_at: NaiveDateTime,
}

impl SiteNews {
    /// Построить запись новости из wire-значений ISS с валидацией.
    pub fn try_new(
        id: i64,
        tag: String,
        title: String,
        published_at: NaiveDateTime,
        modified_at: NaiveDateTime,
    ) -> Result<Self, ParseSiteNewsError> {
        if id <= 0 {
            return Err(ParseSiteNewsError::NonPositiveId(id));
        }
        let id = u64::try_from(id).map_err(|_| ParseSiteNewsError::IdOutOfRange(id))?;

        let tag = tag.trim();
        if tag.is_empty() {
            return Err(ParseSiteNewsError::EmptyTag);
        }
        let title = title.trim();
        if title.is_empty() {
            return Err(ParseSiteNewsError::EmptyTitle);
        }

        Ok(Self {
            id,
            tag: tag.to_owned().into_boxed_str(),
            title: title.to_owned().into_boxed_str(),
            published_at,
            modified_at,
        })
    }

    /// Идентификатор новости (`id`).
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Тег новости (`tag`).
    pub fn tag(&self) -> &str {
        self.tag.as_ref()
    }

    /// Заголовок новости (`title`).
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Время публикации (`published_at`).
    pub fn published_at(&self) -> NaiveDateTime {
        self.published_at
    }

    /// Время изменения (`modified_at`).
    pub fn modified_at(&self) -> NaiveDateTime {
        self.modified_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Событие MOEX ISS (`events`).
pub struct Event {
    id: u64,
    tag: Box<str>,
    title: Box<str>,
    from: Option<NaiveDateTime>,
    modified_at: NaiveDateTime,
}

impl Event {
    /// Построить запись события из wire-значений ISS с валидацией.
    pub fn try_new(
        id: i64,
        tag: String,
        title: String,
        from: Option<NaiveDateTime>,
        modified_at: NaiveDateTime,
    ) -> Result<Self, ParseEventError> {
        if id <= 0 {
            return Err(ParseEventError::NonPositiveId(id));
        }
        let id = u64::try_from(id).map_err(|_| ParseEventError::IdOutOfRange(id))?;

        let tag = tag.trim();
        if tag.is_empty() {
            return Err(ParseEventError::EmptyTag);
        }
        let title = title.trim();
        if title.is_empty() {
            return Err(ParseEventError::EmptyTitle);
        }

        Ok(Self {
            id,
            tag: tag.to_owned().into_boxed_str(),
            title: title.to_owned().into_boxed_str(),
            from,
            modified_at,
        })
    }

    /// Идентификатор события (`id`).
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Тег события (`tag`).
    pub fn tag(&self) -> &str {
        self.tag.as_ref()
    }

    /// Заголовок события (`title`).
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Время начала/актуальности события (`from`), если задано.
    pub fn from(&self) -> Option<NaiveDateTime> {
        self.from
    }

    /// Время изменения (`modified_at`).
    pub fn modified_at(&self) -> NaiveDateTime {
        self.modified_at
    }
}

/// Внутренний набор wire-полей для построения [`SecStat`].
///
/// Отдельная структура снижает вероятность перепутать порядок однотипных
/// аргументов при передаче данных из wire-слоя.
pub(crate) struct SecStatInput {
    pub(crate) secid: String,
    pub(crate) boardid: String,
    pub(crate) voltoday: Option<i64>,
    pub(crate) valtoday: Option<f64>,
    pub(crate) highbid: Option<f64>,
    pub(crate) lowoffer: Option<f64>,
    pub(crate) lastoffer: Option<f64>,
    pub(crate) lastbid: Option<f64>,
    pub(crate) open: Option<f64>,
    pub(crate) low: Option<f64>,
    pub(crate) high: Option<f64>,
    pub(crate) last: Option<f64>,
    pub(crate) numtrades: Option<i64>,
    pub(crate) waprice: Option<f64>,
}

impl SecStat {
    /// Построить запись `secstats` из wire-значений ISS с валидацией инвариантов.
    pub(crate) fn try_new(input: SecStatInput) -> Result<Self, ParseSecStatError> {
        let SecStatInput {
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
        } = input;

        let secid = SecId::try_from(secid)?;
        let boardid = BoardId::try_from(boardid)?;

        let voltoday = match voltoday {
            None => None,
            Some(raw) if raw >= 0 => Some(raw as u64),
            Some(raw) => return Err(ParseSecStatError::NegativeVolToday(raw)),
        };

        let numtrades = match numtrades {
            None => None,
            Some(raw) if raw >= 0 => Some(raw as u64),
            Some(raw) => return Err(ParseSecStatError::NegativeNumTrades(raw)),
        };

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

/// Внутренний набор wire-полей для построения [`HistoryRecord`].
pub(crate) struct HistoryRecordInput {
    pub(crate) boardid: String,
    pub(crate) tradedate: NaiveDate,
    pub(crate) secid: String,
    pub(crate) numtrades: Option<i64>,
    pub(crate) value: Option<f64>,
    pub(crate) open: Option<f64>,
    pub(crate) low: Option<f64>,
    pub(crate) high: Option<f64>,
    pub(crate) close: Option<f64>,
    pub(crate) volume: Option<i64>,
}

impl HistoryRecord {
    /// Построить запись истории из wire-значений ISS с валидацией инвариантов.
    pub(crate) fn try_new(input: HistoryRecordInput) -> Result<Self, ParseHistoryRecordError> {
        let HistoryRecordInput {
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
        } = input;

        let boardid = BoardId::try_from(boardid)?;
        let secid = SecId::try_from(secid)?;

        let numtrades = match numtrades {
            None => None,
            Some(raw) if raw >= 0 => Some(raw as u64),
            Some(raw) => return Err(ParseHistoryRecordError::NegativeNumTrades(raw)),
        };

        let volume = match volume {
            None => None,
            Some(raw) if raw >= 0 => Some(raw as u64),
            Some(raw) => return Err(ParseHistoryRecordError::NegativeVolume(raw)),
        };

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
        })
    }

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
        let short_name = short_name.trim();
        if short_name.is_empty() {
            return Err(ParseIndexError::EmptyShortName);
        }
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
            short_name: short_name.to_owned().into_boxed_str(),
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

/// Внутренний набор wire-полей для построения [`IndexAnalytics`].
pub(crate) struct IndexAnalyticsInput {
    pub(crate) indexid: String,
    pub(crate) tradedate: NaiveDate,
    pub(crate) ticker: String,
    pub(crate) shortnames: String,
    pub(crate) secid: String,
    pub(crate) weight: f64,
    pub(crate) tradingsession: i64,
    pub(crate) trade_session_date: NaiveDate,
}

impl IndexAnalytics {
    /// Построить компонент индекса из wire-значений ISS с валидацией инвариантов.
    pub(crate) fn try_new(input: IndexAnalyticsInput) -> Result<Self, ParseIndexAnalyticsError> {
        let IndexAnalyticsInput {
            indexid,
            tradedate,
            ticker,
            shortnames,
            secid,
            weight,
            tradingsession,
            trade_session_date,
        } = input;

        let indexid = IndexId::try_from(indexid)?;
        let ticker = SecId::try_from(ticker).map_err(ParseIndexAnalyticsError::InvalidTicker)?;
        let secid = SecId::try_from(secid).map_err(ParseIndexAnalyticsError::InvalidSecId)?;

        let shortnames = shortnames.trim();
        if shortnames.is_empty() {
            return Err(ParseIndexAnalyticsError::EmptyShortnames);
        }
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
            shortnames: shortnames.to_owned().into_boxed_str(),
            secid,
            weight,
            tradingsession: tradingsession as u8,
            trade_session_date,
        })
    }

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

/// Итератор по «актуальным» индексам: с максимальной датой `till`.
pub fn actual_indexes(indexes: &[Index]) -> impl Iterator<Item = &Index> {
    let latest_till = indexes.iter().filter_map(Index::till).max();
    indexes
        .iter()
        .filter(move |index| index.till() == latest_till)
}

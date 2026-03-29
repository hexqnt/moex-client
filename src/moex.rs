//! Клиент для HTTP-взаимодействия с ISS API и ошибки транспортного уровня.

#[cfg(any(feature = "async", feature = "blocking"))]
mod client;
mod constants;
mod convert;
pub mod decode;
mod payload;
mod wire;

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

#[cfg(any(feature = "async", feature = "blocking"))]
use reqwest::{StatusCode, header::HeaderMap};
use thiserror::Error;

use crate::models::{
    BoardId, EngineName, IndexId, MarketName, ParseBoardError, ParseCandleBorderError,
    ParseCandleError, ParseEngineError, ParseEventError, ParseHistoryDatesError,
    ParseHistoryRecordError, ParseIndexAnalyticsError, ParseIndexError, ParseMarketError,
    ParseOrderbookError, ParseSecStatError, ParseSecurityBoardError, ParseSecurityError,
    ParseSecuritySnapshotError, ParseSiteNewsError, ParseTradeError, ParseTurnoverError, SecId,
};

/// Асинхронный ленивый paginator по страницам `history`.
#[cfg(all(feature = "async", feature = "history"))]
pub use client::AsyncHistoryPages;
/// Блокирующий ленивый paginator по страницам `history`.
#[cfg(all(feature = "blocking", feature = "history"))]
pub use client::HistoryPages;
/// Асинхронные HTTP-клиенты ISS API.
#[cfg(feature = "async")]
pub use client::{
    AsyncCandlesPages, AsyncGlobalSecuritiesPages, AsyncIndexAnalyticsPages,
    AsyncMarketSecuritiesPages, AsyncMarketTradesPages, AsyncMoexClient, AsyncMoexClientBuilder,
    AsyncOwnedBoardScope, AsyncOwnedEngineScope, AsyncOwnedIndexScope, AsyncOwnedMarketScope,
    AsyncOwnedMarketSecurityScope, AsyncOwnedSecurityResourceScope, AsyncOwnedSecurityScope,
    AsyncRawIssRequestBuilder, AsyncSecStatsPages, AsyncSecuritiesPages, AsyncTradesPages,
};
#[cfg(all(feature = "async", feature = "news"))]
/// Асинхронные paginator-ы новостных endpoint-ов.
pub use client::{AsyncEventsPages, AsyncSiteNewsPages};
/// Блокирующие HTTP-клиенты ISS API.
#[cfg(feature = "blocking")]
pub use client::{
    CandlesPages, GlobalSecuritiesPages, IndexAnalyticsPages, MarketSecuritiesPages,
    MarketTradesPages, OwnedBoardScope, OwnedEngineScope, OwnedIndexScope, OwnedMarketScope,
    OwnedMarketSecurityScope, OwnedSecurityResourceScope, OwnedSecurityScope, RawIssRequestBuilder,
    SecStatsPages, SecuritiesPages, TradesPages,
};
#[cfg(all(feature = "blocking", feature = "news"))]
/// Блокирующие paginator-ы новостных endpoint-ов.
pub use client::{EventsPages, SiteNewsPages};

/// Явное имя блокирующего ISS-клиента.
#[cfg(feature = "blocking")]
pub type BlockingMoexClient = client::BlockingMoexClient;
/// Явное имя builder-а блокирующего ISS-клиента.
#[cfg(feature = "blocking")]
pub type BlockingMoexClientBuilder = client::BlockingMoexClientBuilder;

/// Типизированный идентификатор ISS endpoint-а для raw-запросов.
///
/// Позволяет строить raw-запросы без ручной сборки path-строк.
#[derive(Debug, Clone, Copy)]
pub enum IssEndpoint<'a> {
    /// `/iss/statistics/engines/stock/markets/index/analytics.json` (`indices`).
    Indexes,
    /// `/iss/statistics/engines/stock/markets/index/analytics/{indexid}.json` (`analytics`).
    IndexAnalytics { indexid: &'a IndexId },
    /// `/iss/turnovers.json` (`turnovers`).
    Turnovers,
    /// `/iss/engines/{engine}/turnovers.json` (`turnovers`).
    EngineTurnovers { engine: &'a EngineName },
    /// `/iss/engines.json` (`engines`).
    Engines,
    /// `/iss/engines/{engine}/markets.json` (`markets`).
    Markets { engine: &'a EngineName },
    /// `/iss/engines/{engine}/markets/{market}/boards.json` (`boards`).
    Boards {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/securities.json` (`securities`).
    GlobalSecurities,
    /// `/iss/securities/{secid}.json` (`securities`).
    SecurityInfo { security: &'a SecId },
    /// `/iss/securities/{secid}.json` (`boards`).
    SecurityBoards { security: &'a SecId },
    /// `/iss/engines/{engine}/markets/{market}/securities.json` (`securities`).
    MarketSecurities {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/securities/{secid}.json` (`securities`).
    MarketSecurityInfo {
        engine: &'a EngineName,
        market: &'a MarketName,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/orderbook.json` (`orderbook`).
    MarketOrderbook {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/trades.json` (`trades`).
    MarketTrades {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/secstats.json` (`secstats`).
    SecStats {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities.json` (`securities`).
    Securities {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities.json` (`securities,marketdata`).
    BoardSecuritySnapshots {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/orderbook.json` (`orderbook`).
    Orderbook {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/trades.json` (`trades`).
    Trades {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/candles.json` (`candles`).
    Candles {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/securities/{secid}/candleborders.json` (`borders`).
    CandleBorders {
        engine: &'a EngineName,
        market: &'a MarketName,
        security: &'a SecId,
    },
    /// `/iss/sitenews.json` (`sitenews`).
    #[cfg(feature = "news")]
    SiteNews,
    /// `/iss/events.json` (`events`).
    #[cfg(feature = "news")]
    Events,
    /// `/iss/history/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/dates.json` (`dates`).
    #[cfg(feature = "history")]
    HistoryDates {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/history/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}.json` (`history`).
    #[cfg(feature = "history")]
    History {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
}

impl IssEndpoint<'_> {
    /// Построить относительный endpoint-path (`*.json`) для raw-запроса.
    pub fn path(self) -> String {
        match self {
            Self::Indexes => constants::INDEXES_ENDPOINT.to_owned(),
            Self::IndexAnalytics { indexid } => constants::index_analytics_endpoint(indexid),
            Self::Turnovers => constants::TURNOVERS_ENDPOINT.to_owned(),
            Self::EngineTurnovers { engine } => constants::engine_turnovers_endpoint(engine),
            Self::Engines => constants::ENGINES_ENDPOINT.to_owned(),
            Self::Markets { engine } => constants::markets_endpoint(engine),
            Self::Boards { engine, market } => constants::boards_endpoint(engine, market),
            Self::GlobalSecurities => constants::GLOBAL_SECURITIES_ENDPOINT.to_owned(),
            Self::SecurityInfo { security } | Self::SecurityBoards { security } => {
                constants::security_endpoint(security)
            }
            Self::MarketSecurities { engine, market } => {
                constants::market_securities_endpoint(engine, market)
            }
            Self::MarketSecurityInfo {
                engine,
                market,
                security,
            } => constants::market_security_endpoint(engine, market, security),
            Self::MarketOrderbook { engine, market } => {
                constants::market_orderbook_endpoint(engine, market)
            }
            Self::MarketTrades { engine, market } => {
                constants::market_trades_endpoint(engine, market)
            }
            Self::SecStats { engine, market } => constants::secstats_endpoint(engine, market),
            Self::Securities {
                engine,
                market,
                board,
            }
            | Self::BoardSecuritySnapshots {
                engine,
                market,
                board,
            } => constants::securities_endpoint(engine, market, board),
            Self::Orderbook {
                engine,
                market,
                board,
                security,
            } => constants::orderbook_endpoint(engine, market, board, security),
            Self::Trades {
                engine,
                market,
                board,
                security,
            } => constants::trades_endpoint(engine, market, board, security),
            Self::Candles {
                engine,
                market,
                board,
                security,
            } => constants::candles_endpoint(engine, market, board, security),
            Self::CandleBorders {
                engine,
                market,
                security,
            } => constants::candleborders_endpoint(engine, market, security),
            #[cfg(feature = "news")]
            Self::SiteNews => constants::SITENEWS_ENDPOINT.to_owned(),
            #[cfg(feature = "news")]
            Self::Events => constants::EVENTS_ENDPOINT.to_owned(),
            #[cfg(feature = "history")]
            Self::HistoryDates {
                engine,
                market,
                board,
                security,
            } => constants::history_dates_endpoint(engine, market, board, security),
            #[cfg(feature = "history")]
            Self::History {
                engine,
                market,
                board,
                security,
            } => constants::history_endpoint(engine, market, board, security),
        }
    }

    /// Таблица по умолчанию для `iss.only`, если endpoint описывает единственную цель выборки.
    pub fn default_table(self) -> Option<&'static str> {
        match self {
            Self::Indexes => Some("indices"),
            Self::IndexAnalytics { .. } => Some("analytics"),
            Self::Turnovers | Self::EngineTurnovers { .. } => Some("turnovers"),
            Self::Engines => Some("engines"),
            Self::Markets { .. } => Some("markets"),
            Self::Boards { .. } | Self::SecurityBoards { .. } => Some("boards"),
            Self::GlobalSecurities
            | Self::SecurityInfo { .. }
            | Self::MarketSecurities { .. }
            | Self::MarketSecurityInfo { .. }
            | Self::Securities { .. } => Some("securities"),
            Self::BoardSecuritySnapshots { .. } => Some("securities,marketdata"),
            Self::Orderbook { .. } | Self::MarketOrderbook { .. } => Some("orderbook"),
            Self::Trades { .. } | Self::MarketTrades { .. } => Some("trades"),
            Self::Candles { .. } => Some("candles"),
            Self::CandleBorders { .. } => Some("borders"),
            Self::SecStats { .. } => Some("secstats"),
            #[cfg(feature = "news")]
            Self::SiteNews => Some("sitenews"),
            #[cfg(feature = "news")]
            Self::Events => Some("events"),
            #[cfg(feature = "history")]
            Self::HistoryDates { .. } => Some("dates"),
            #[cfg(feature = "history")]
            Self::History { .. } => Some("history"),
        }
    }
}

/// Политика повторных попыток для операций с [`MoexError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    max_attempts: NonZeroU32,
    delay: Duration,
}

impl RetryPolicy {
    /// Создать политику повторов с числом попыток и delay по умолчанию.
    ///
    /// Значение delay по умолчанию — `400ms`.
    pub fn new(max_attempts: NonZeroU32) -> Self {
        Self {
            max_attempts,
            delay: Duration::from_millis(400),
        }
    }

    /// Установить фиксированную паузу между попытками.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Максимальное число попыток (включая первую).
    pub fn max_attempts(self) -> NonZeroU32 {
        self.max_attempts
    }

    /// Пауза между попытками.
    pub fn delay(self) -> Duration {
        self.delay
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(NonZeroU32::new(3).expect("retry policy default attempts must be non-zero"))
    }
}

/// Выполнить blocking-операцию с повтором retryable-ошибок.
///
/// Повтор выполняется только для [`MoexError::is_retryable`].
pub fn with_retry<T, F>(policy: RetryPolicy, mut action: F) -> Result<T, MoexError>
where
    F: FnMut() -> Result<T, MoexError>,
{
    let mut attempts_left = policy.max_attempts().get();
    loop {
        match action() {
            Ok(value) => return Ok(value),
            Err(error) if attempts_left > 1 && error.is_retryable() => {
                attempts_left -= 1;
                std::thread::sleep(policy.delay());
            }
            Err(error) => return Err(error),
        }
    }
}

/// Выполнить async-операцию с повтором retryable-ошибок.
///
/// `sleep` задаётся вызывающим кодом, чтобы библиотека не навязывала runtime.
#[cfg(feature = "async")]
pub async fn with_retry_async<T, F, Fut, S, SleepFut>(
    policy: RetryPolicy,
    mut action: F,
    mut sleep: S,
) -> Result<T, MoexError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, MoexError>>,
    S: FnMut(Duration) -> SleepFut,
    SleepFut: std::future::Future<Output = ()>,
{
    let mut attempts_left = policy.max_attempts().get();
    loop {
        match action().await {
            Ok(value) => return Ok(value),
            Err(error) if attempts_left > 1 && error.is_retryable() => {
                attempts_left -= 1;
                sleep(policy.delay()).await;
            }
            Err(error) => return Err(error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Ограничение частоты запросов.
///
/// Хранит минимальный интервал между последовательными запросами.
pub struct RateLimit {
    min_interval: Duration,
}

impl RateLimit {
    /// Создать limit из минимального интервала между запросами.
    pub fn every(min_interval: Duration) -> Self {
        Self { min_interval }
    }

    /// Создать limit из числа запросов в секунду.
    ///
    /// Интервал округляется вверх до целого числа наносекунд.
    pub fn per_second(requests_per_second: NonZeroU32) -> Self {
        let per_second_nanos: u128 = 1_000_000_000;
        let requests = u128::from(requests_per_second.get());
        let nanos = per_second_nanos.div_ceil(requests);
        let nanos = u64::try_from(nanos).unwrap_or(u64::MAX);
        Self::every(Duration::from_nanos(nanos))
    }

    /// Минимальный интервал между запросами.
    pub fn min_interval(self) -> Duration {
        self.min_interval
    }
}

#[derive(Debug, Clone)]
/// Состояние rate-limit для последовательности запросов.
pub struct RateLimiter {
    limit: RateLimit,
    next_allowed_at: Option<Instant>,
}

impl RateLimiter {
    /// Создать новый limiter с заданным ограничением.
    pub fn new(limit: RateLimit) -> Self {
        Self {
            limit,
            next_allowed_at: None,
        }
    }

    /// Текущая конфигурация ограничения.
    pub fn limit(&self) -> RateLimit {
        self.limit
    }

    /// Рассчитать задержку до следующего запроса и зарезервировать слот.
    pub fn reserve_delay(&mut self) -> Duration {
        self.reserve_delay_at(Instant::now())
    }

    fn reserve_delay_at(&mut self, now: Instant) -> Duration {
        let scheduled_at = match self.next_allowed_at {
            Some(next_allowed_at) if next_allowed_at > now => next_allowed_at,
            _ => now,
        };
        let delay = scheduled_at.saturating_duration_since(now);
        self.next_allowed_at = Some(scheduled_at + self.limit.min_interval);
        delay
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Универсальный переключатель ISS-параметров со значениями `on/off`.
pub enum IssToggle {
    /// Значение `off`.
    #[default]
    Off,
    /// Значение `on`.
    On,
}

impl IssToggle {
    /// Вернуть wire-значение параметра (`on`/`off`).
    pub const fn as_query_value(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::On => "on",
        }
    }
}

impl From<bool> for IssToggle {
    fn from(value: bool) -> Self {
        if value { Self::On } else { Self::Off }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Системные опции ISS-запроса (`iss.*`) для raw endpoint-ов.
pub struct IssRequestOptions {
    metadata: Option<IssToggle>,
    data: Option<IssToggle>,
    version: Option<IssToggle>,
    json: Option<Box<str>>,
}

impl IssRequestOptions {
    /// Создать пустой набор опций.
    pub fn new() -> Self {
        Self::default()
    }

    /// Установить `iss.meta`.
    pub fn metadata(mut self, metadata: IssToggle) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Установить `iss.data`.
    pub fn data(mut self, data: IssToggle) -> Self {
        self.data = Some(data);
        self
    }

    /// Установить `iss.version`.
    pub fn version(mut self, version: IssToggle) -> Self {
        self.version = Some(version);
        self
    }

    /// Установить `iss.json`.
    pub fn json(mut self, json: impl Into<String>) -> Self {
        self.json = Some(json.into().into_boxed_str());
        self
    }

    /// Текущее значение `iss.meta`, если задано.
    pub fn metadata_value(&self) -> Option<IssToggle> {
        self.metadata
    }

    /// Текущее значение `iss.data`, если задано.
    pub fn data_value(&self) -> Option<IssToggle> {
        self.data
    }

    /// Текущее значение `iss.version`, если задано.
    pub fn version_value(&self) -> Option<IssToggle> {
        self.version
    }

    /// Текущее значение `iss.json`, если задано.
    pub fn json_value(&self) -> Option<&str> {
        self.json.as_deref()
    }
}

#[derive(Debug, Clone)]
/// HTTP-ответ raw ISS-запроса без дополнительной валидации статуса/формата.
#[cfg(any(feature = "async", feature = "blocking"))]
pub struct RawIssResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

#[cfg(any(feature = "async", feature = "blocking"))]
impl RawIssResponse {
    pub(crate) fn new(status: StatusCode, headers: HeaderMap, body: String) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// HTTP-статус ответа.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// HTTP-заголовки ответа.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Полное тело ответа как строка.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Разобрать ответ на части (`status`, `headers`, `body`).
    pub fn into_parts(self) -> (StatusCode, HeaderMap, String) {
        (self.status, self.headers, self.body)
    }
}

/// Выполнить blocking-операцию c соблюдением [`RateLimiter`].
pub fn with_rate_limit<T, F>(limiter: &mut RateLimiter, action: F) -> T
where
    F: FnOnce() -> T,
{
    let delay = limiter.reserve_delay();
    if !delay.is_zero() {
        std::thread::sleep(delay);
    }
    action()
}

/// Выполнить async-операцию c соблюдением [`RateLimiter`].
///
/// `sleep` задаётся приложением, чтобы библиотека не требовала конкретный runtime.
#[cfg(feature = "async")]
pub async fn with_rate_limit_async<T, F, Fut, S, SleepFut>(
    limiter: &mut RateLimiter,
    action: F,
    mut sleep: S,
) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
    S: FnMut(Duration) -> SleepFut,
    SleepFut: std::future::Future<Output = ()>,
{
    let delay = limiter.reserve_delay();
    if !delay.is_zero() {
        sleep(delay).await;
    }
    action().await
}

#[derive(Debug, Error)]
/// Ошибки выполнения запросов к ISS и конвертации wire-ответов в доменные типы.
pub enum MoexError {
    /// Некорректно задан базовый URL ISS.
    #[error("invalid base URL '{base_url}': {reason}")]
    InvalidBaseUrl {
        /// Строка базового URL, которую не удалось разобрать.
        base_url: &'static str,
        /// Подробность ошибки парсинга URL.
        reason: String,
    },
    /// Ошибка сборки `reqwest::blocking::Client`.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("failed to build HTTP client: {source}")]
    BuildHttpClient {
        /// Исходная ошибка HTTP-клиента.
        #[source]
        source: reqwest::Error,
    },
    /// Для async rate-limit не задана функция `sleep`.
    #[error(
        "async rate limit requires sleep function; set AsyncMoexClientBuilder::rate_limit_sleep(...)"
    )]
    MissingAsyncRateLimitSleep,
    /// Не удалось построить URL конкретного endpoint.
    #[error("failed to build URL for endpoint '{endpoint}': {reason}")]
    EndpointUrl {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Подробность ошибки построения URL.
        reason: String,
    },
    /// Для raw-запроса не задан endpoint-path.
    #[error("raw request path is not set")]
    MissingRawPath,
    /// Некорректно задан endpoint-path в raw-запросе.
    #[error("invalid raw request path '{path}': {reason}")]
    InvalidRawPath {
        /// Исходный endpoint-path.
        path: Box<str>,
        /// Деталь ошибки валидации path.
        reason: Box<str>,
    },
    /// В raw JSON-ответе отсутствует запрошенная таблица ISS.
    #[error("raw endpoint '{endpoint}' does not contain table '{table}'")]
    MissingRawTable {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS (`history`, `trades`, `securities` и т.д.).
        table: Box<str>,
    },
    /// Строка raw-таблицы содержит число значений, отличное от числа колонок.
    #[error(
        "raw table '{table}' from endpoint '{endpoint}' has invalid row width at row {row}: expected {expected}, got {actual}"
    )]
    InvalidRawTableRowWidth {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS.
        table: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Число колонок таблицы.
        expected: usize,
        /// Число значений в конкретной строке.
        actual: usize,
    },
    /// Не удалось декодировать строку raw-таблицы в пользовательский тип.
    #[error("failed to decode raw table '{table}' row {row} from endpoint '{endpoint}': {source}")]
    InvalidRawTableRow {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS.
        table: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Исходная ошибка JSON-декодера.
        #[source]
        source: serde_json::Error,
    },
    /// Ошибка отправки HTTP-запроса до получения ответа.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("request to endpoint '{endpoint}' failed: {source}")]
    Request {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Исходная ошибка HTTP-клиента.
        #[source]
        source: reqwest::Error,
    },
    /// Endpoint вернул HTTP-статус вне диапазона `2xx`.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error(
        "endpoint '{endpoint}' returned HTTP {status} (content-type={content_type:?}, prefix={body_prefix:?})"
    )]
    HttpStatus {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// HTTP-статус ответа.
        status: StatusCode,
        /// Значение HTTP `content-type`, если присутствует.
        content_type: Option<Box<str>>,
        /// Начало тела ответа для диагностики.
        body_prefix: Box<str>,
    },
    /// Ошибка чтения тела HTTP-ответа.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("failed to read endpoint '{endpoint}' response body: {source}")]
    ReadBody {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Исходная ошибка HTTP-клиента.
        #[source]
        source: reqwest::Error,
    },
    /// Ошибка десериализации JSON-пейлоада ISS.
    #[error("failed to decode endpoint '{endpoint}' JSON payload: {source}")]
    Decode {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Исходная ошибка JSON-декодера.
        #[source]
        source: serde_json::Error,
    },
    /// Endpoint вернул payload, не похожий на JSON.
    #[error(
        "endpoint '{endpoint}' returned non-JSON payload (content-type={content_type:?}, prefix={body_prefix:?})"
    )]
    NonJsonPayload {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Значение HTTP `content-type`, если присутствует.
        content_type: Option<Box<str>>,
        /// Начало тела ответа для диагностики.
        body_prefix: Box<str>,
    },
    /// В endpoint `securities/{secid}` пришло больше одной строки `securities`.
    #[error("endpoint '{endpoint}' returned unexpected security rows count: {row_count}")]
    UnexpectedSecurityRows {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Фактическое число строк в таблице `securities`.
        row_count: usize,
    },
    /// В history endpoint `.../dates` пришло больше одной строки `dates`.
    #[error("endpoint '{endpoint}' returned unexpected history dates rows count: {row_count}")]
    UnexpectedHistoryDatesRows {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Фактическое число строк в таблице `dates`.
        row_count: usize,
    },
    /// Ошибка преобразования строки таблицы `indices`.
    #[error("invalid index row {row} from endpoint '{endpoint}': {source}")]
    InvalidIndex {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseIndexError,
    },
    /// Ошибка преобразования строки таблицы `dates` из history endpoint.
    #[error("invalid history dates row {row} from endpoint '{endpoint}': {source}")]
    InvalidHistoryDates {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseHistoryDatesError,
    },
    /// Ошибка преобразования строки таблицы `history`.
    #[error("invalid history row {row} from endpoint '{endpoint}': {source}")]
    InvalidHistory {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseHistoryRecordError,
    },
    /// Ошибка преобразования строки таблицы `turnovers`.
    #[error("invalid turnover row {row} from endpoint '{endpoint}': {source}")]
    InvalidTurnover {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseTurnoverError,
    },
    /// Ошибка преобразования строки таблицы `sitenews`.
    #[error("invalid sitenews row {row} from endpoint '{endpoint}': {source}")]
    InvalidSiteNews {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSiteNewsError,
    },
    /// Ошибка преобразования строки таблицы `events`.
    #[error("invalid events row {row} from endpoint '{endpoint}': {source}")]
    InvalidEvent {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseEventError,
    },
    /// Ошибка преобразования строки таблицы `secstats`.
    #[error("invalid secstats row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecStat {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecStatError,
    },
    /// Ошибка преобразования строки таблицы `analytics`.
    #[error("invalid index analytics row {row} from endpoint '{endpoint}': {source}")]
    InvalidIndexAnalytics {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseIndexAnalyticsError,
    },
    /// Ошибка преобразования строки таблицы `engines`.
    #[error("invalid engine row {row} from endpoint '{endpoint}': {source}")]
    InvalidEngine {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseEngineError,
    },
    /// Ошибка преобразования строки таблицы `markets`.
    #[error("invalid market row {row} from endpoint '{endpoint}': {source}")]
    InvalidMarket {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseMarketError,
    },
    /// Ошибка преобразования строки таблицы `boards`.
    #[error("invalid board row {row} from endpoint '{endpoint}': {source}")]
    InvalidBoard {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseBoardError,
    },
    /// Ошибка преобразования строки таблицы `boards` в endpoint `securities/{secid}`.
    #[error("invalid security board row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecurityBoard {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecurityBoardError,
    },
    /// Ошибка преобразования строки таблицы `securities`.
    #[error("invalid security row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecurity {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecurityError,
    },
    /// Ошибка преобразования строки таблиц `securities`/`marketdata` в снимок инструмента.
    #[error("invalid security snapshot {table} row {row} from endpoint '{endpoint}': {source}")]
    InvalidSecuritySnapshot {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Имя таблицы ISS (`securities` или `marketdata`).
        table: &'static str,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseSecuritySnapshotError,
    },
    /// Ошибка преобразования строки таблицы `orderbook`.
    #[error("invalid orderbook row {row} from endpoint '{endpoint}': {source}")]
    InvalidOrderbook {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseOrderbookError,
    },
    /// Ошибка преобразования строки таблицы `borders`.
    #[error("invalid candle border row {row} from endpoint '{endpoint}': {source}")]
    InvalidCandleBorder {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseCandleBorderError,
    },
    /// Ошибка преобразования строки таблицы `candles`.
    #[error("invalid candle row {row} from endpoint '{endpoint}': {source}")]
    InvalidCandle {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseCandleError,
    },
    /// Ошибка преобразования строки таблицы `trades`.
    #[error("invalid trade row {row} from endpoint '{endpoint}': {source}")]
    InvalidTrade {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Индекс строки в таблице ISS.
        row: usize,
        /// Деталь ошибки парсинга доменной сущности.
        #[source]
        source: ParseTradeError,
    },
    /// Переполнение счётчика `start` при авто-пагинации ISS.
    #[error(
        "pagination overflow for endpoint '{endpoint}': start={start}, limit={limit} exceeds u32"
    )]
    PaginationOverflow {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Текущее значение `start`.
        start: u32,
        /// Размер страницы `limit`.
        limit: u32,
    },
    /// Обнаружен зацикленный ответ при авто-пагинации ISS.
    #[error(
        "pagination is stuck for endpoint '{endpoint}': repeated page at start={start}, limit={limit}"
    )]
    PaginationStuck {
        /// Относительный путь endpoint.
        endpoint: Box<str>,
        /// Текущее значение `start`.
        start: u32,
        /// Размер страницы `limit`.
        limit: u32,
    },
}

impl MoexError {
    /// Признак, что операцию обычно имеет смысл повторить с backoff.
    pub fn is_retryable(&self) -> bool {
        match self {
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::BuildHttpClient { .. } => false,
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::Request { source, .. } => {
                source.is_timeout()
                    || source.is_connect()
                    || source.status().is_some_and(is_retryable_status)
            }
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::ReadBody { .. } => true,
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::HttpStatus { status, .. } => is_retryable_status(*status),
            _ => false,
        }
    }

    /// HTTP-статус, если ошибка была получена после ответа сервера.
    #[cfg(any(feature = "async", feature = "blocking"))]
    pub fn status_code(&self) -> Option<StatusCode> {
        match self {
            Self::Request { source, .. } => source.status(),
            Self::HttpStatus { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Диагностический префикс тела ответа, если он сохранён в ошибке.
    pub fn response_body_prefix(&self) -> Option<&str> {
        match self {
            #[cfg(any(feature = "async", feature = "blocking"))]
            Self::HttpStatus { body_prefix, .. } | Self::NonJsonPayload { body_prefix, .. } => {
                Some(body_prefix)
            }
            #[cfg(not(any(feature = "async", feature = "blocking")))]
            Self::NonJsonPayload { body_prefix, .. } => Some(body_prefix),
            _ => None,
        }
    }
}

#[cfg(any(feature = "async", feature = "blocking"))]
fn is_retryable_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepeatPagePolicy {
    Error,
}

#[cfg(test)]
mod tests;

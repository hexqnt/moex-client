use std::num::NonZeroU32;
#[cfg(any(feature = "blocking", feature = "async"))]
use std::sync::Mutex;
use std::time::Duration;

#[cfg(feature = "blocking")]
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::{Url, header::HeaderMap};

use crate::models::{
    Board, BoardId, Candle, CandleBorder, CandleQuery, Engine, EngineName, Index, IndexAnalytics,
    IndexId, Market, MarketName, OrderbookLevel, PageRequest, Pagination, ParseBoardIdError,
    ParseEngineNameError, ParseIndexError, ParseMarketNameError, ParseSecIdError, SecId, SecStat,
    Security, SecurityBoard, SecuritySnapshot, Trade, Turnover,
};
#[cfg(feature = "news")]
use crate::models::{Event, SiteNews};
#[cfg(feature = "history")]
use crate::models::{HistoryDates, HistoryRecord};

use super::constants::*;
use super::payload::{
    decode_board_security_snapshots_json_with_endpoint, decode_boards_json_with_endpoint,
    decode_candle_borders_json_with_endpoint, decode_candles_json_with_endpoint,
    decode_engines_json_payload, decode_index_analytics_json_with_endpoint,
    decode_indexes_json_payload, decode_markets_json_with_endpoint,
    decode_orderbook_json_with_endpoint, decode_raw_table_rows_json_with_endpoint,
    decode_secstats_json_with_endpoint, decode_securities_json_with_endpoint,
    decode_security_boards_json_with_endpoint, decode_trades_json_with_endpoint,
    decode_turnovers_json_with_endpoint,
};
#[cfg(feature = "news")]
use super::payload::{decode_events_json_with_endpoint, decode_sitenews_json_with_endpoint};
#[cfg(feature = "history")]
use super::payload::{decode_history_dates_json_with_endpoint, decode_history_json_with_endpoint};
use super::{
    IssEndpoint, IssRequestOptions, IssToggle, MoexError, RawIssResponse, RepeatPagePolicy,
};
#[cfg(any(feature = "blocking", feature = "async"))]
use super::{RateLimit, RateLimiter};

/// Блокирующий клиент ISS API Московской биржи.
///
/// Клиент хранит базовый URL, режим выдачи `iss.meta` и переиспользуемый
/// экземпляр `reqwest::blocking::Client`.
#[cfg(feature = "blocking")]
pub struct BlockingMoexClient {
    base_url: Url,
    metadata: bool,
    client: Client,
    rate_limiter: Option<Mutex<RateLimiter>>,
}

/// Builder для конфигурации [`BlockingMoexClient`].
#[cfg(feature = "blocking")]
pub struct BlockingMoexClientBuilder {
    base_url: Option<Url>,
    metadata: bool,
    client: Option<Client>,
    http_client: ClientBuilder,
    rate_limit: Option<RateLimit>,
}

/// Асинхронный клиент ISS API Московской биржи.
///
/// Клиент хранит базовый URL, режим выдачи `iss.meta` и переиспользуемый
/// экземпляр `reqwest::Client`.
#[cfg(feature = "async")]
pub struct AsyncMoexClient {
    base_url: Url,
    metadata: bool,
    client: reqwest::Client,
    rate_limit: Option<AsyncRateLimitState>,
}

/// Builder для конфигурации [`AsyncMoexClient`].
#[cfg(feature = "async")]
pub struct AsyncMoexClientBuilder {
    base_url: Option<Url>,
    metadata: bool,
    client: Option<reqwest::Client>,
    http_client: reqwest::ClientBuilder,
    rate_limit: Option<RateLimit>,
    rate_limit_sleep: Option<AsyncRateLimitSleep>,
}

#[cfg(feature = "async")]
type AsyncSleepFuture = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static>>;

#[cfg(feature = "async")]
type AsyncRateLimitSleep = std::sync::Arc<dyn Fn(Duration) -> AsyncSleepFuture + Send + Sync>;

#[cfg(feature = "async")]
struct AsyncRateLimitState {
    limiter: Mutex<RateLimiter>,
    sleep: AsyncRateLimitSleep,
}

/// Универсальный builder для произвольных ISS endpoint-ов.
///
/// Служит low-level escape hatch для endpoint-ов, которые не покрыты
/// строгим high-level API.
#[cfg(feature = "blocking")]
pub struct RawIssRequestBuilder<'a> {
    client: &'a BlockingMoexClient,
    path: Option<Box<str>>,
    query: Vec<(Box<str>, Box<str>)>,
}

/// Асинхронный универсальный builder для произвольных ISS endpoint-ов.
#[cfg(feature = "async")]
pub struct AsyncRawIssRequestBuilder<'a> {
    client: &'a AsyncMoexClient,
    path: Option<Box<str>>,
    query: Vec<(Box<str>, Box<str>)>,
}

/// Асинхронный ленивый paginator по страницам `index_analytics`.
#[cfg(feature = "async")]
pub struct AsyncIndexAnalyticsPages<'a> {
    client: &'a AsyncMoexClient,
    indexid: &'a IndexId,
    pagination: PaginationTracker<(chrono::NaiveDate, SecId)>,
}

/// Асинхронный ленивый paginator по страницам `securities`.
#[cfg(feature = "async")]
pub struct AsyncSecuritiesPages<'a> {
    client: &'a AsyncMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    pagination: PaginationTracker<SecId>,
}

/// Асинхронный ленивый paginator по страницам глобального `securities`.
#[cfg(feature = "async")]
pub struct AsyncGlobalSecuritiesPages<'a> {
    client: &'a AsyncMoexClient,
    pagination: PaginationTracker<SecId>,
}

/// Асинхронный ленивый paginator по страницам `sitenews`.
#[cfg(all(feature = "async", feature = "news"))]
pub struct AsyncSiteNewsPages<'a> {
    client: &'a AsyncMoexClient,
    pagination: PaginationTracker<u64>,
}

/// Асинхронный ленивый paginator по страницам `events`.
#[cfg(all(feature = "async", feature = "news"))]
pub struct AsyncEventsPages<'a> {
    client: &'a AsyncMoexClient,
    pagination: PaginationTracker<u64>,
}

/// Асинхронный ленивый paginator по страницам market-level `securities`.
#[cfg(feature = "async")]
pub struct AsyncMarketSecuritiesPages<'a> {
    client: &'a AsyncMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    pagination: PaginationTracker<SecId>,
}

/// Асинхронный ленивый paginator по страницам market-level `trades`.
#[cfg(feature = "async")]
pub struct AsyncMarketTradesPages<'a> {
    client: &'a AsyncMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    pagination: PaginationTracker<u64>,
}

/// Асинхронный ленивый paginator по страницам `trades`.
#[cfg(feature = "async")]
pub struct AsyncTradesPages<'a> {
    client: &'a AsyncMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    security: &'a SecId,
    pagination: PaginationTracker<u64>,
}

/// Асинхронный ленивый paginator по страницам `history`.
#[cfg(all(feature = "async", feature = "history"))]
pub struct AsyncHistoryPages<'a> {
    client: &'a AsyncMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    security: &'a SecId,
    pagination: PaginationTracker<chrono::NaiveDate>,
}

/// Асинхронный ленивый paginator по страницам `secstats`.
#[cfg(feature = "async")]
pub struct AsyncSecStatsPages<'a> {
    client: &'a AsyncMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    pagination: PaginationTracker<(SecId, BoardId)>,
}

/// Асинхронный ленивый paginator по страницам `candles`.
#[cfg(feature = "async")]
pub struct AsyncCandlesPages<'a> {
    client: &'a AsyncMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    security: &'a SecId,
    query: CandleQuery,
    pagination: PaginationTracker<chrono::NaiveDateTime>,
}

/// Ленивый paginator по страницам `index_analytics`.
#[cfg(feature = "blocking")]
pub struct IndexAnalyticsPages<'a> {
    client: &'a BlockingMoexClient,
    indexid: &'a IndexId,
    pagination: PaginationTracker<(chrono::NaiveDate, SecId)>,
}

/// Ленивый paginator по страницам `securities`.
#[cfg(feature = "blocking")]
pub struct SecuritiesPages<'a> {
    client: &'a BlockingMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    pagination: PaginationTracker<SecId>,
}

/// Ленивый paginator по страницам глобального `securities`.
#[cfg(feature = "blocking")]
pub struct GlobalSecuritiesPages<'a> {
    client: &'a BlockingMoexClient,
    pagination: PaginationTracker<SecId>,
}

/// Ленивый paginator по страницам `sitenews`.
#[cfg(all(feature = "blocking", feature = "news"))]
pub struct SiteNewsPages<'a> {
    client: &'a BlockingMoexClient,
    pagination: PaginationTracker<u64>,
}

/// Ленивый paginator по страницам `events`.
#[cfg(all(feature = "blocking", feature = "news"))]
pub struct EventsPages<'a> {
    client: &'a BlockingMoexClient,
    pagination: PaginationTracker<u64>,
}

/// Ленивый paginator по страницам market-level `securities`.
#[cfg(feature = "blocking")]
pub struct MarketSecuritiesPages<'a> {
    client: &'a BlockingMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    pagination: PaginationTracker<SecId>,
}

/// Ленивый paginator по страницам market-level `trades`.
#[cfg(feature = "blocking")]
pub struct MarketTradesPages<'a> {
    client: &'a BlockingMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    pagination: PaginationTracker<u64>,
}

/// Ленивый paginator по страницам `trades`.
#[cfg(feature = "blocking")]
pub struct TradesPages<'a> {
    client: &'a BlockingMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    security: &'a SecId,
    pagination: PaginationTracker<u64>,
}

/// Ленивый paginator по страницам `history`.
#[cfg(all(feature = "blocking", feature = "history"))]
pub struct HistoryPages<'a> {
    client: &'a BlockingMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    security: &'a SecId,
    pagination: PaginationTracker<chrono::NaiveDate>,
}

/// Ленивый paginator по страницам `secstats`.
#[cfg(feature = "blocking")]
pub struct SecStatsPages<'a> {
    client: &'a BlockingMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    pagination: PaginationTracker<(SecId, BoardId)>,
}

/// Ленивый paginator по страницам `candles`.
#[cfg(feature = "blocking")]
pub struct CandlesPages<'a> {
    client: &'a BlockingMoexClient,
    engine: &'a EngineName,
    market: &'a MarketName,
    board: &'a BoardId,
    security: &'a SecId,
    query: CandleQuery,
    pagination: PaginationTracker<chrono::NaiveDateTime>,
}

#[derive(Clone)]
/// Асинхронный scope по `indexid`, владеющий значением.
///
/// Полезен для ergonomic-chain, где вход передаётся как `impl TryInto<IndexId>`.
#[cfg(feature = "async")]
pub struct AsyncOwnedIndexScope<'a> {
    client: &'a AsyncMoexClient,
    indexid: IndexId,
}

#[derive(Clone)]
/// Асинхронный scope по `engine`, владеющий значением.
#[cfg(feature = "async")]
pub struct AsyncOwnedEngineScope<'a> {
    client: &'a AsyncMoexClient,
    engine: EngineName,
}

#[derive(Clone)]
/// Асинхронный scope по `engine/market`, владеющий значениями.
#[cfg(feature = "async")]
pub struct AsyncOwnedMarketScope<'a> {
    client: &'a AsyncMoexClient,
    engine: EngineName,
    market: MarketName,
}

#[derive(Clone)]
/// Асинхронный scope по `engine/market/security`, владеющий значениями.
#[cfg(feature = "async")]
pub struct AsyncOwnedMarketSecurityScope<'a> {
    client: &'a AsyncMoexClient,
    engine: EngineName,
    market: MarketName,
    security: SecId,
}

#[derive(Clone)]
/// Асинхронный scope по `engine/market/board`, владеющий значениями.
#[cfg(feature = "async")]
pub struct AsyncOwnedBoardScope<'a> {
    client: &'a AsyncMoexClient,
    engine: EngineName,
    market: MarketName,
    board: BoardId,
}

#[derive(Clone)]
/// Асинхронный scope по `securities/{secid}`, владеющий `secid`.
#[cfg(feature = "async")]
pub struct AsyncOwnedSecurityResourceScope<'a> {
    client: &'a AsyncMoexClient,
    security: SecId,
}

#[derive(Clone)]
/// Асинхронный scope по `engine/market/board/security`, владеющий значениями.
#[cfg(feature = "async")]
pub struct AsyncOwnedSecurityScope<'a> {
    client: &'a AsyncMoexClient,
    engine: EngineName,
    market: MarketName,
    board: BoardId,
    security: SecId,
}

#[derive(Clone)]
/// Blocking scope по `indexid`, владеющий значением.
///
/// Полезен для ergonomic-chain, где вход передаётся как `impl TryInto<IndexId>`.
#[cfg(feature = "blocking")]
pub struct OwnedIndexScope<'a> {
    client: &'a BlockingMoexClient,
    indexid: IndexId,
}

#[derive(Clone)]
/// Blocking scope по `engine`, владеющий значением.
#[cfg(feature = "blocking")]
pub struct OwnedEngineScope<'a> {
    client: &'a BlockingMoexClient,
    engine: EngineName,
}

#[derive(Clone)]
/// Blocking scope по `engine/market`, владеющий значениями.
#[cfg(feature = "blocking")]
pub struct OwnedMarketScope<'a> {
    client: &'a BlockingMoexClient,
    engine: EngineName,
    market: MarketName,
}

#[derive(Clone)]
/// Blocking scope по `engine/market/security`, владеющий значениями.
#[cfg(feature = "blocking")]
pub struct OwnedMarketSecurityScope<'a> {
    client: &'a BlockingMoexClient,
    engine: EngineName,
    market: MarketName,
    security: SecId,
}

#[derive(Clone)]
/// Blocking scope по `engine/market/board`, владеющий значениями.
#[cfg(feature = "blocking")]
pub struct OwnedBoardScope<'a> {
    client: &'a BlockingMoexClient,
    engine: EngineName,
    market: MarketName,
    board: BoardId,
}

#[derive(Clone)]
/// Blocking scope по `securities/{secid}`, владеющий `secid`.
#[cfg(feature = "blocking")]
pub struct OwnedSecurityResourceScope<'a> {
    client: &'a BlockingMoexClient,
    security: SecId,
}

#[derive(Clone)]
/// Blocking scope по `engine/market/board/security`, владеющий значениями.
#[cfg(feature = "blocking")]
pub struct OwnedSecurityScope<'a> {
    client: &'a BlockingMoexClient,
    engine: EngineName,
    market: MarketName,
    board: BoardId,
    security: SecId,
}

struct PaginationTracker<K> {
    endpoint: Box<str>,
    page_limit: NonZeroU32,
    repeat_page_policy: RepeatPagePolicy,
    start: u32,
    first_key_on_previous_page: Option<K>,
    finished: bool,
}

enum PaginationAdvance {
    YieldPage,
    EndOfPages,
}

#[cfg(any(feature = "blocking", feature = "async"))]
fn resolve_base_url_or_default(base_url: Option<Url>) -> Result<Url, MoexError> {
    match base_url {
        Some(base_url) => Ok(base_url),
        None => Url::parse(BASE_URL).map_err(|source| MoexError::InvalidBaseUrl {
            base_url: BASE_URL,
            reason: source.to_string(),
        }),
    }
}

#[cfg(feature = "blocking")]
fn resolve_blocking_http_client(
    client: Option<Client>,
    http_client: ClientBuilder,
) -> Result<Client, MoexError> {
    match client {
        Some(client) => Ok(client),
        None => http_client
            .build()
            .map_err(|source| MoexError::BuildHttpClient { source }),
    }
}

#[cfg(feature = "async")]
fn resolve_async_http_client(
    client: Option<reqwest::Client>,
    http_client: reqwest::ClientBuilder,
) -> Result<reqwest::Client, MoexError> {
    match client {
        Some(client) => Ok(client),
        None => http_client
            .build()
            .map_err(|source| MoexError::BuildHttpClient { source }),
    }
}

#[cfg(feature = "async")]
fn resolve_async_rate_limit_state(
    rate_limit: Option<RateLimit>,
    rate_limit_sleep: Option<AsyncRateLimitSleep>,
) -> Result<Option<AsyncRateLimitState>, MoexError> {
    match rate_limit {
        Some(limit) => {
            let sleep = rate_limit_sleep.ok_or(MoexError::MissingAsyncRateLimitSleep)?;
            Ok(Some(AsyncRateLimitState {
                limiter: Mutex::new(RateLimiter::new(limit)),
                sleep,
            }))
        }
        None => Ok(None),
    }
}

#[cfg(feature = "blocking")]
impl BlockingMoexClientBuilder {
    /// Включить или отключить выдачу `iss.meta`.
    pub fn metadata(mut self, metadata: bool) -> Self {
        self.metadata = metadata;
        self
    }

    /// Задать явный базовый URL ISS.
    pub fn base_url(mut self, base_url: Url) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Передать готовый `reqwest::blocking::Client`.
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Установить общий таймаут HTTP-запросов.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http_client = self.http_client.timeout(timeout);
        self
    }

    /// Установить таймаут установления TCP-соединения.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.http_client = self.http_client.connect_timeout(timeout);
        self
    }

    /// Установить заголовок `User-Agent` для всех запросов.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.http_client = self.http_client.user_agent(user_agent.into());
        self
    }

    /// Установить `User-Agent` в формате `{crate_name}/{crate_version}`.
    pub fn user_agent_from_crate(self) -> Self {
        self.user_agent(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
    }

    /// Установить набор заголовков по умолчанию для всех запросов.
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.http_client = self.http_client.default_headers(headers);
        self
    }

    /// Добавить proxy для HTTP-клиента.
    ///
    /// Метод можно вызывать несколько раз, если требуется набор правил proxy-маршрутизации.
    pub fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
        self.http_client = self.http_client.proxy(proxy);
        self
    }

    /// Отключить использование proxy из окружения и системных настроек.
    pub fn no_proxy(mut self) -> Self {
        self.http_client = self.http_client.no_proxy();
        self
    }

    /// Включить ограничение частоты запросов на уровне клиента.
    ///
    /// Лимит применяется ко всем endpoint-методам и raw-запросам этого экземпляра клиента.
    pub fn rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Построить блокирующий клиент ISS.
    pub fn build(self) -> Result<BlockingMoexClient, MoexError> {
        let Self {
            base_url,
            metadata,
            client,
            http_client,
            rate_limit,
        } = self;
        let base_url = resolve_base_url_or_default(base_url)?;
        let client = resolve_blocking_http_client(client, http_client)?;
        Ok(BlockingMoexClient::with_base_url_and_rate_limit(
            client, base_url, metadata, rate_limit,
        ))
    }
}

#[cfg(feature = "async")]
impl AsyncMoexClientBuilder {
    /// Включить или отключить выдачу `iss.meta`.
    pub fn metadata(mut self, metadata: bool) -> Self {
        self.metadata = metadata;
        self
    }

    /// Задать явный базовый URL ISS.
    pub fn base_url(mut self, base_url: Url) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Передать готовый `reqwest::Client`.
    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Установить общий таймаут HTTP-запросов.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http_client = self.http_client.timeout(timeout);
        self
    }

    /// Установить таймаут установления TCP-соединения.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.http_client = self.http_client.connect_timeout(timeout);
        self
    }

    /// Установить заголовок `User-Agent` для всех запросов.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.http_client = self.http_client.user_agent(user_agent.into());
        self
    }

    /// Установить `User-Agent` в формате `{crate_name}/{crate_version}`.
    pub fn user_agent_from_crate(self) -> Self {
        self.user_agent(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
    }

    /// Установить набор заголовков по умолчанию для всех запросов.
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.http_client = self.http_client.default_headers(headers);
        self
    }

    /// Добавить proxy для HTTP-клиента.
    ///
    /// Метод можно вызывать несколько раз, если требуется набор правил proxy-маршрутизации.
    pub fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
        self.http_client = self.http_client.proxy(proxy);
        self
    }

    /// Отключить использование proxy из окружения и системных настроек.
    pub fn no_proxy(mut self) -> Self {
        self.http_client = self.http_client.no_proxy();
        self
    }

    /// Включить ограничение частоты запросов на уровне клиента.
    ///
    /// Для применения задержек нужно дополнительно передать `sleep` через
    /// [`Self::rate_limit_sleep`].
    pub fn rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Задать async-функцию ожидания для использования с [`Self::rate_limit`].
    ///
    /// Обычно это функция runtime-а, например `tokio::time::sleep`.
    pub fn rate_limit_sleep<F, Fut>(mut self, sleep: F) -> Self
    where
        F: Fn(Duration) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        self.rate_limit_sleep = Some(std::sync::Arc::new(move |delay| Box::pin(sleep(delay))));
        self
    }

    /// Построить асинхронный клиент ISS.
    pub fn build(self) -> Result<AsyncMoexClient, MoexError> {
        let Self {
            base_url,
            metadata,
            client,
            http_client,
            rate_limit,
            rate_limit_sleep,
        } = self;
        let base_url = resolve_base_url_or_default(base_url)?;
        let client = resolve_async_http_client(client, http_client)?;
        let rate_limit = resolve_async_rate_limit_state(rate_limit, rate_limit_sleep)?;
        Ok(AsyncMoexClient::with_base_url_and_rate_limit(
            client, base_url, metadata, rate_limit,
        ))
    }
}

#[cfg(feature = "blocking")]
impl BlockingMoexClient {
    /// Создать builder для конфигурации клиента ISS.
    pub fn builder() -> BlockingMoexClientBuilder {
        BlockingMoexClientBuilder {
            base_url: None,
            metadata: false,
            client: None,
            http_client: Client::builder(),
            rate_limit: None,
        }
    }

    /// Создать клиент с базовым URL ISS по умолчанию (`iss.meta=off`).
    pub fn new() -> Result<Self, MoexError> {
        Self::builder().build()
    }

    /// Создать клиент с базовым URL ISS по умолчанию и `iss.meta=on`.
    pub fn new_with_metadata() -> Result<Self, MoexError> {
        Self::builder().metadata(true).build()
    }

    /// Создать клиент на базе переданного `reqwest`-клиента (`iss.meta=off`).
    ///
    /// Позволяет переиспользовать настройки таймаутов, прокси и TLS.
    pub fn with_client(client: Client) -> Result<Self, MoexError> {
        Self::builder().client(client).build()
    }

    /// Создать клиент на базе переданного `reqwest`-клиента и включить `iss.meta`.
    pub fn with_client_with_metadata(client: Client) -> Result<Self, MoexError> {
        Self::builder().metadata(true).client(client).build()
    }

    /// Создать клиент с явным базовым URL и готовым HTTP-клиентом (`iss.meta=off`).
    pub fn with_base_url(client: Client, base_url: Url) -> Self {
        Self::with_base_url_and_rate_limit(client, base_url, false, None)
    }

    /// Создать клиент с явным базовым URL, готовым HTTP-клиентом и `iss.meta=on`.
    pub fn with_base_url_with_metadata(client: Client, base_url: Url) -> Self {
        Self::with_base_url_and_rate_limit(client, base_url, true, None)
    }

    /// Текущее ограничение частоты запросов, если оно включено.
    pub fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limiter
            .as_ref()
            .map(|limiter| lock_rate_limiter(limiter).limit())
    }

    fn with_base_url_and_rate_limit(
        client: Client,
        base_url: Url,
        metadata: bool,
        rate_limit: Option<RateLimit>,
    ) -> Self {
        Self {
            base_url,
            metadata,
            client,
            rate_limiter: rate_limit.map(|limit| Mutex::new(RateLimiter::new(limit))),
        }
    }

    /// Создать raw-builder для произвольного ISS endpoint.
    pub fn raw(&self) -> RawIssRequestBuilder<'_> {
        RawIssRequestBuilder {
            client: self,
            path: None,
            query: Vec::new(),
        }
    }

    /// Создать raw-builder для типизированного ISS endpoint-а.
    ///
    /// Builder автоматически получает `path` и значение `iss.only` по умолчанию.
    pub fn raw_endpoint(&self, endpoint: IssEndpoint<'_>) -> RawIssRequestBuilder<'_> {
        let path = endpoint.path();
        let request = self.raw().path(path);
        match endpoint.default_table() {
            Some(table) => request.only(table),
            None => request,
        }
    }

    /// Получить список индексов из таблицы `indices`.
    pub fn indexes(&self) -> Result<Vec<Index>, MoexError> {
        let payload = self.get_payload(
            INDEXES_ENDPOINT,
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "indices"),
                (INDICES_COLUMNS_PARAM, INDICES_COLUMNS),
            ],
        )?;
        decode_indexes_json_payload(&payload)
    }

    /// Получить состав индекса (`analytics`) с единым режимом выборки страниц.
    pub fn index_analytics_query(
        &self,
        indexid: &IndexId,
        page_request: PageRequest,
    ) -> Result<Vec<IndexAnalytics>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_index_analytics_page(indexid, Pagination::default())
            }
            PageRequest::Page(pagination) => self.fetch_index_analytics_page(indexid, pagination),
            PageRequest::All { page_limit } => {
                self.index_analytics_pages(indexid, page_limit).all()
            }
        }
    }

    /// Создать ленивый paginator страниц `index_analytics`.
    pub fn index_analytics_pages<'a>(
        &'a self,
        indexid: &'a IndexId,
        page_limit: NonZeroU32,
    ) -> IndexAnalyticsPages<'a> {
        IndexAnalyticsPages {
            client: self,
            indexid,
            pagination: PaginationTracker::new(
                index_analytics_endpoint(indexid),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить обороты ISS (`/iss/turnovers`).
    pub fn turnovers(&self) -> Result<Vec<Turnover>, MoexError> {
        let payload = self.get_payload(
            TURNOVERS_ENDPOINT,
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "turnovers"),
                (TURNOVERS_COLUMNS_PARAM, TURNOVERS_COLUMNS),
            ],
        )?;
        decode_turnovers_json_with_endpoint(&payload, TURNOVERS_ENDPOINT)
    }

    /// Получить обороты ISS по движку (`/iss/engines/{engine}/turnovers`).
    pub fn engine_turnovers(&self, engine: &EngineName) -> Result<Vec<Turnover>, MoexError> {
        let endpoint = engine_turnovers_endpoint(engine);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "turnovers"),
                (TURNOVERS_COLUMNS_PARAM, TURNOVERS_COLUMNS),
            ],
        )?;
        decode_turnovers_json_with_endpoint(&payload, endpoint.as_str())
    }

    #[cfg(feature = "news")]
    /// Получить новости ISS (`sitenews`) с единым режимом выборки страниц.
    pub fn sitenews_query(&self, page_request: PageRequest) -> Result<Vec<SiteNews>, MoexError> {
        match page_request {
            PageRequest::FirstPage => self.fetch_sitenews_page(Pagination::default()),
            PageRequest::Page(pagination) => self.fetch_sitenews_page(pagination),
            PageRequest::All { page_limit } => self.sitenews_pages(page_limit).all(),
        }
    }

    #[cfg(feature = "news")]
    /// Создать ленивый paginator страниц `sitenews`.
    pub fn sitenews_pages<'a>(&'a self, page_limit: NonZeroU32) -> SiteNewsPages<'a> {
        SiteNewsPages {
            client: self,
            pagination: PaginationTracker::new(
                SITENEWS_ENDPOINT,
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    #[cfg(feature = "news")]
    /// Получить события ISS (`events`) с единым режимом выборки страниц.
    pub fn events_query(&self, page_request: PageRequest) -> Result<Vec<Event>, MoexError> {
        match page_request {
            PageRequest::FirstPage => self.fetch_events_page(Pagination::default()),
            PageRequest::Page(pagination) => self.fetch_events_page(pagination),
            PageRequest::All { page_limit } => self.events_pages(page_limit).all(),
        }
    }

    #[cfg(feature = "news")]
    /// Создать ленивый paginator страниц `events`.
    pub fn events_pages<'a>(&'a self, page_limit: NonZeroU32) -> EventsPages<'a> {
        EventsPages {
            client: self,
            pagination: PaginationTracker::new(
                EVENTS_ENDPOINT,
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить `secstats` с единым режимом выборки страниц.
    pub fn secstats_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        page_request: PageRequest,
    ) -> Result<Vec<SecStat>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_secstats_page(engine, market, Pagination::default())
            }
            PageRequest::Page(pagination) => self.fetch_secstats_page(engine, market, pagination),
            PageRequest::All { page_limit } => {
                self.secstats_pages(engine, market, page_limit).all()
            }
        }
    }

    /// Создать ленивый paginator страниц `secstats`.
    pub fn secstats_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        page_limit: NonZeroU32,
    ) -> SecStatsPages<'a> {
        SecStatsPages {
            client: self,
            engine,
            market,
            pagination: PaginationTracker::new(
                secstats_endpoint(engine, market),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить доступные торговые движки ISS (`engines`).
    pub fn engines(&self) -> Result<Vec<Engine>, MoexError> {
        let payload = self.get_payload(
            ENGINES_ENDPOINT,
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "engines"),
                (ENGINES_COLUMNS_PARAM, ENGINES_COLUMNS),
            ],
        )?;
        decode_engines_json_payload(&payload)
    }

    /// Получить рынки (`markets`) для заданного движка.
    pub fn markets(&self, engine: &EngineName) -> Result<Vec<Market>, MoexError> {
        let endpoint = markets_endpoint(engine);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "markets"),
                (MARKETS_COLUMNS_PARAM, MARKETS_COLUMNS),
            ],
        )?;
        decode_markets_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить режимы торгов (`boards`) для пары движок/рынок.
    pub fn boards(
        &self,
        engine: &EngineName,
        market: &MarketName,
    ) -> Result<Vec<Board>, MoexError> {
        let endpoint = boards_endpoint(engine, market);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "boards"),
                (BOARDS_COLUMNS_PARAM, BOARDS_COLUMNS),
            ],
        )?;
        decode_boards_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить режимы торгов инструмента (`boards`) из endpoint `securities/{secid}`.
    pub fn security_boards(&self, security: &SecId) -> Result<Vec<SecurityBoard>, MoexError> {
        let endpoint = security_boards_endpoint(security);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "boards"),
                (BOARDS_COLUMNS_PARAM, SECURITY_BOARDS_COLUMNS),
            ],
        )?;
        decode_security_boards_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить карточку инструмента (`securities`) из endpoint `securities/{secid}`.
    ///
    /// Возвращает `Ok(None)`, если таблица `securities` пустая.
    pub fn security_info(&self, security: &SecId) -> Result<Option<Security>, MoexError> {
        let endpoint = security_endpoint(security);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "securities"),
                (SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS),
            ],
        )?;
        let securities = decode_securities_json_with_endpoint(&payload, endpoint.as_str())?;
        optional_single_security(endpoint.as_str(), securities)
    }

    #[cfg(feature = "history")]
    /// Получить диапазон доступных исторических дат по инструменту и board.
    ///
    /// Возвращает `Ok(None)`, если таблица `dates` пустая.
    pub fn history_dates(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
    ) -> Result<Option<HistoryDates>, MoexError> {
        let endpoint = history_dates_endpoint(engine, market, board, security);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "dates"),
            ],
        )?;
        let dates = decode_history_dates_json_with_endpoint(&payload, endpoint.as_str())?;
        optional_single_history_dates(endpoint.as_str(), dates)
    }

    #[cfg(feature = "history")]
    /// Получить исторические данные (`history`) с единым режимом выборки страниц.
    pub fn history_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        page_request: PageRequest,
    ) -> Result<Vec<HistoryRecord>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_history_page(engine, market, board, security, Pagination::default())
            }
            PageRequest::Page(pagination) => {
                self.fetch_history_page(engine, market, board, security, pagination)
            }
            PageRequest::All { page_limit } => self
                .history_pages(engine, market, board, security, page_limit)
                .all(),
        }
    }

    #[cfg(feature = "history")]
    /// Создать ленивый paginator страниц `history`.
    pub fn history_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
        page_limit: NonZeroU32,
    ) -> HistoryPages<'a> {
        HistoryPages {
            client: self,
            engine,
            market,
            board,
            security,
            pagination: PaginationTracker::new(
                history_endpoint(engine, market, board, security),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить снимки инструментов (`LOTSIZE` и `LAST`) для режима торгов.
    pub fn board_snapshots(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
    ) -> Result<Vec<SecuritySnapshot>, MoexError> {
        let endpoint = securities_endpoint(engine, market, board);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "securities,marketdata"),
                (SECURITIES_COLUMNS_PARAM, SECURITIES_SNAPSHOT_COLUMNS),
                (MARKETDATA_COLUMNS_PARAM, MARKETDATA_LAST_COLUMNS),
            ],
        )?;
        decode_board_security_snapshots_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить снимки инструментов (`LOTSIZE` и `LAST`) по данным `SecurityBoard`.
    pub fn board_security_snapshots(
        &self,
        board: &SecurityBoard,
    ) -> Result<Vec<SecuritySnapshot>, MoexError> {
        self.board_snapshots(board.engine(), board.market(), board.boardid())
    }

    /// Зафиксировать owning-scope по `engine` из ergonomic-входа.
    pub fn engine<E>(&self, engine: E) -> Result<OwnedEngineScope<'_>, ParseEngineNameError>
    where
        E: TryInto<EngineName>,
        E::Error: Into<ParseEngineNameError>,
    {
        let engine = engine.try_into().map_err(Into::into)?;
        Ok(OwnedEngineScope {
            client: self,
            engine,
        })
    }

    /// Shortcut для часто используемого engine `stock`.
    pub fn stock(&self) -> Result<OwnedEngineScope<'_>, ParseEngineNameError> {
        self.engine("stock")
    }

    /// Зафиксировать owning-scope по `indexid` из ergonomic-входа.
    pub fn index<I>(&self, indexid: I) -> Result<OwnedIndexScope<'_>, ParseIndexError>
    where
        I: TryInto<IndexId>,
        I::Error: Into<ParseIndexError>,
    {
        let indexid = indexid.try_into().map_err(Into::into)?;
        Ok(OwnedIndexScope {
            client: self,
            indexid,
        })
    }

    /// Зафиксировать owning-scope по `secid` из ergonomic-входа.
    pub fn security<S>(
        &self,
        security: S,
    ) -> Result<OwnedSecurityResourceScope<'_>, ParseSecIdError>
    where
        S: TryInto<SecId>,
        S::Error: Into<ParseSecIdError>,
    {
        let security = security.try_into().map_err(Into::into)?;
        Ok(OwnedSecurityResourceScope {
            client: self,
            security,
        })
    }

    /// Получить глобальный список инструментов (`/iss/securities`) с единым режимом выборки страниц.
    pub fn global_securities_query(
        &self,
        page_request: PageRequest,
    ) -> Result<Vec<Security>, MoexError> {
        match page_request {
            PageRequest::FirstPage => self.fetch_global_securities_page(Pagination::default()),
            PageRequest::Page(pagination) => self.fetch_global_securities_page(pagination),
            PageRequest::All { page_limit } => self.global_securities_pages(page_limit).all(),
        }
    }

    /// Создать ленивый paginator страниц глобального `securities`.
    pub fn global_securities_pages<'a>(
        &'a self,
        page_limit: NonZeroU32,
    ) -> GlobalSecuritiesPages<'a> {
        GlobalSecuritiesPages {
            client: self,
            pagination: PaginationTracker::new(
                GLOBAL_SECURITIES_ENDPOINT,
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить карточку инструмента на уровне рынка (`.../markets/{market}/securities/{secid}`).
    ///
    /// Возвращает `Ok(None)`, если endpoint не содержит строк `securities`.
    pub fn market_security_info(
        &self,
        engine: &EngineName,
        market: &MarketName,
        security: &SecId,
    ) -> Result<Option<Security>, MoexError> {
        let endpoint = market_security_endpoint(engine, market, security);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "securities"),
                (SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS),
            ],
        )?;
        let securities = decode_securities_json_with_endpoint(&payload, endpoint.as_str())?;
        optional_single_security(endpoint.as_str(), securities)
    }

    /// Получить инструменты (`securities`) на уровне рынка с единым режимом выборки страниц.
    pub fn market_securities_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        page_request: PageRequest,
    ) -> Result<Vec<Security>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_market_securities_page(engine, market, Pagination::default())
            }
            PageRequest::Page(pagination) => {
                self.fetch_market_securities_page(engine, market, pagination)
            }
            PageRequest::All { page_limit } => self
                .market_securities_pages(engine, market, page_limit)
                .all(),
        }
    }

    /// Создать ленивый paginator страниц market-level `securities`.
    pub fn market_securities_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        page_limit: NonZeroU32,
    ) -> MarketSecuritiesPages<'a> {
        MarketSecuritiesPages {
            client: self,
            engine,
            market,
            pagination: PaginationTracker::new(
                market_securities_endpoint(engine, market),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить market-level стакан (`orderbook`) по первой странице ISS.
    pub fn market_orderbook(
        &self,
        engine: &EngineName,
        market: &MarketName,
    ) -> Result<Vec<OrderbookLevel>, MoexError> {
        let endpoint = market_orderbook_endpoint(engine, market);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "orderbook"),
                (ORDERBOOK_COLUMNS_PARAM, ORDERBOOK_COLUMNS),
            ],
        )?;
        decode_orderbook_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить доступные границы свечей (`candleborders`) по инструменту.
    pub fn candle_borders(
        &self,
        engine: &EngineName,
        market: &MarketName,
        security: &SecId,
    ) -> Result<Vec<CandleBorder>, MoexError> {
        let endpoint = candleborders_endpoint(engine, market, security);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[(ISS_META_PARAM, metadata_value(self.metadata))],
        )?;
        decode_candle_borders_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить market-level сделки (`trades`) с единым режимом выборки страниц.
    pub fn market_trades_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        page_request: PageRequest,
    ) -> Result<Vec<Trade>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_market_trades_page(engine, market, Pagination::default())
            }
            PageRequest::Page(pagination) => {
                self.fetch_market_trades_page(engine, market, pagination)
            }
            PageRequest::All { page_limit } => {
                self.market_trades_pages(engine, market, page_limit).all()
            }
        }
    }

    /// Создать ленивый paginator страниц market-level `trades`.
    pub fn market_trades_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        page_limit: NonZeroU32,
    ) -> MarketTradesPages<'a> {
        MarketTradesPages {
            client: self,
            engine,
            market,
            pagination: PaginationTracker::new(
                market_trades_endpoint(engine, market),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить инструменты (`securities`) с единым режимом выборки страниц.
    ///
    /// `PageRequest::FirstPage` — только первая страница,
    /// `PageRequest::Page` — явные `start`/`limit`,
    /// `PageRequest::All` — полная выгрузка с авто-пагинацией.
    pub fn securities_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        page_request: PageRequest,
    ) -> Result<Vec<Security>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_securities_page(engine, market, board, Pagination::default())
            }
            PageRequest::Page(pagination) => {
                self.fetch_securities_page(engine, market, board, pagination)
            }
            PageRequest::All { page_limit } => self
                .securities_pages(engine, market, board, page_limit)
                .all(),
        }
    }

    /// Создать ленивый paginator страниц `securities`.
    pub fn securities_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        page_limit: NonZeroU32,
    ) -> SecuritiesPages<'a> {
        SecuritiesPages {
            client: self,
            engine,
            market,
            board,
            pagination: PaginationTracker::new(
                securities_endpoint(engine, market, board),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить текущий стакан (`orderbook`) по инструменту.
    pub fn orderbook(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
    ) -> Result<Vec<OrderbookLevel>, MoexError> {
        let endpoint = orderbook_endpoint(engine, market, board, security);
        let payload = self.get_payload(
            endpoint.as_str(),
            &[
                (ISS_META_PARAM, metadata_value(self.metadata)),
                (ISS_ONLY_PARAM, "orderbook"),
                (ORDERBOOK_COLUMNS_PARAM, ORDERBOOK_COLUMNS),
            ],
        )?;
        decode_orderbook_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить свечи (`candles`) с единым режимом выборки страниц.
    pub fn candles_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        query: CandleQuery,
        page_request: PageRequest,
    ) -> Result<Vec<Candle>, MoexError> {
        match page_request {
            PageRequest::FirstPage => self.fetch_candles_page(
                engine,
                market,
                board,
                security,
                query,
                Pagination::default(),
            ),
            PageRequest::Page(pagination) => {
                self.fetch_candles_page(engine, market, board, security, query, pagination)
            }
            PageRequest::All { page_limit } => self
                .candles_pages(engine, market, board, security, query, page_limit)
                .all(),
        }
    }

    /// Создать ленивый paginator страниц `candles`.
    pub fn candles_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
        query: CandleQuery,
        page_limit: NonZeroU32,
    ) -> CandlesPages<'a> {
        CandlesPages {
            client: self,
            engine,
            market,
            board,
            security,
            query,
            pagination: PaginationTracker::new(
                candles_endpoint(engine, market, board, security),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить сделки (`trades`) с единым режимом выборки страниц.
    pub fn trades_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        page_request: PageRequest,
    ) -> Result<Vec<Trade>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_trades_page(engine, market, board, security, Pagination::default())
            }
            PageRequest::Page(pagination) => {
                self.fetch_trades_page(engine, market, board, security, pagination)
            }
            PageRequest::All { page_limit } => self
                .trades_pages(engine, market, board, security, page_limit)
                .all(),
        }
    }

    /// Создать ленивый paginator страниц `trades`.
    pub fn trades_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
        page_limit: NonZeroU32,
    ) -> TradesPages<'a> {
        TradesPages {
            client: self,
            engine,
            market,
            board,
            security,
            pagination: PaginationTracker::new(
                trades_endpoint(engine, market, board, security),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    fn fetch_securities_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        pagination: Pagination,
    ) -> Result<Vec<Security>, MoexError> {
        let endpoint = securities_endpoint(engine, market, board);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "securities")
                .append_pair(SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_securities_json_with_endpoint(&payload, endpoint.as_str())
    }

    fn fetch_global_securities_page(
        &self,
        pagination: Pagination,
    ) -> Result<Vec<Security>, MoexError> {
        let endpoint = GLOBAL_SECURITIES_ENDPOINT;
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "securities")
                .append_pair(SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint, endpoint_url)?;
        decode_securities_json_with_endpoint(&payload, endpoint)
    }

    #[cfg(feature = "news")]
    fn fetch_sitenews_page(&self, pagination: Pagination) -> Result<Vec<SiteNews>, MoexError> {
        let endpoint = SITENEWS_ENDPOINT;
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "sitenews")
                .append_pair(SITENEWS_COLUMNS_PARAM, SITENEWS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint, endpoint_url)?;
        decode_sitenews_json_with_endpoint(&payload, endpoint)
    }

    #[cfg(feature = "news")]
    fn fetch_events_page(&self, pagination: Pagination) -> Result<Vec<Event>, MoexError> {
        let endpoint = EVENTS_ENDPOINT;
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "events")
                .append_pair(EVENTS_COLUMNS_PARAM, EVENTS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint, endpoint_url)?;
        decode_events_json_with_endpoint(&payload, endpoint)
    }

    fn fetch_market_securities_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        pagination: Pagination,
    ) -> Result<Vec<Security>, MoexError> {
        let endpoint = market_securities_endpoint(engine, market);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "securities")
                .append_pair(SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_securities_json_with_endpoint(&payload, endpoint.as_str())
    }

    fn fetch_market_trades_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        pagination: Pagination,
    ) -> Result<Vec<Trade>, MoexError> {
        let endpoint = market_trades_endpoint(engine, market);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "trades")
                .append_pair(TRADES_COLUMNS_PARAM, TRADES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_trades_json_with_endpoint(&payload, endpoint.as_str())
    }

    fn fetch_secstats_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        pagination: Pagination,
    ) -> Result<Vec<SecStat>, MoexError> {
        let endpoint = secstats_endpoint(engine, market);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "secstats")
                .append_pair(SECSTATS_COLUMNS_PARAM, SECSTATS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_secstats_json_with_endpoint(&payload, endpoint.as_str())
    }

    fn fetch_index_analytics_page(
        &self,
        indexid: &IndexId,
        pagination: Pagination,
    ) -> Result<Vec<IndexAnalytics>, MoexError> {
        let endpoint = index_analytics_endpoint(indexid);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "analytics")
                .append_pair(ANALYTICS_COLUMNS_PARAM, ANALYTICS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_index_analytics_json_with_endpoint(&payload, endpoint.as_str())
    }

    fn fetch_candles_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        query: CandleQuery,
        pagination: Pagination,
    ) -> Result<Vec<Candle>, MoexError> {
        let endpoint = candles_endpoint(engine, market, board, security);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query_pairs = endpoint_url.query_pairs_mut();
            query_pairs
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "candles")
                .append_pair(CANDLES_COLUMNS_PARAM, CANDLES_COLUMNS);
        }
        append_candle_query_to_url(&mut endpoint_url, query);
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_candles_json_with_endpoint(&payload, endpoint.as_str())
    }

    fn fetch_trades_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        pagination: Pagination,
    ) -> Result<Vec<Trade>, MoexError> {
        let endpoint = trades_endpoint(engine, market, board, security);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "trades")
                .append_pair(TRADES_COLUMNS_PARAM, TRADES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_trades_json_with_endpoint(&payload, endpoint.as_str())
    }

    #[cfg(feature = "history")]
    fn fetch_history_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        pagination: Pagination,
    ) -> Result<Vec<HistoryRecord>, MoexError> {
        let endpoint = history_endpoint(engine, market, board, security);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "history")
                .append_pair(HISTORY_COLUMNS_PARAM, HISTORY_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url)?;
        decode_history_json_with_endpoint(&payload, endpoint.as_str())
    }

    #[cfg(test)]
    pub(super) fn collect_paginated<T, K, F, G>(
        endpoint: &str,
        page_limit: NonZeroU32,
        repeat_page_policy: RepeatPagePolicy,
        mut fetch_page: F,
        first_key_of: G,
    ) -> Result<Vec<T>, MoexError>
    where
        F: FnMut(Pagination) -> Result<Vec<T>, MoexError>,
        G: Fn(&T) -> K,
        K: Eq,
    {
        let mut pagination = PaginationTracker::new(endpoint, page_limit, repeat_page_policy);
        let mut items = Vec::new();

        while let Some(paging) = pagination.next_page_request() {
            let page = fetch_page(paging)?;
            let first_key_on_page = page.first().map(&first_key_of);
            match pagination.advance(page.len(), first_key_on_page)? {
                PaginationAdvance::YieldPage => items.extend(page),
                PaginationAdvance::EndOfPages => break,
            }
        }

        Ok(items)
    }

    fn endpoint_url(&self, endpoint: &str) -> Result<Url, MoexError> {
        self.base_url
            .join(endpoint)
            .map_err(|source| MoexError::EndpointUrl {
                endpoint: endpoint.to_owned().into_boxed_str(),
                reason: source.to_string(),
            })
    }

    fn get_payload(
        &self,
        endpoint: &str,
        query_params: &[(&'static str, &'static str)],
    ) -> Result<String, MoexError> {
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut url_query = endpoint_url.query_pairs_mut();
            for (key, value) in query_params {
                url_query.append_pair(key, value);
            }
        }
        self.fetch_payload(endpoint, endpoint_url)
    }

    fn fetch_payload(&self, endpoint: &str, endpoint_url: Url) -> Result<String, MoexError> {
        self.wait_for_rate_limit();
        let response =
            self.client
                .get(endpoint_url)
                .send()
                .map_err(|source| MoexError::Request {
                    endpoint: endpoint.to_owned().into_boxed_str(),
                    source,
                })?;
        let status = response.status();

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_owned().into_boxed_str());

        let payload = response.text().map_err(|source| MoexError::ReadBody {
            endpoint: endpoint.to_owned().into_boxed_str(),
            source,
        })?;

        if !status.is_success() {
            return Err(MoexError::HttpStatus {
                endpoint: endpoint.to_owned().into_boxed_str(),
                status,
                content_type,
                body_prefix: truncate_prefix(&payload, NON_JSON_BODY_PREFIX_CHARS),
            });
        }

        if !looks_like_json_payload(content_type.as_deref(), &payload) {
            return Err(MoexError::NonJsonPayload {
                endpoint: endpoint.to_owned().into_boxed_str(),
                content_type,
                body_prefix: truncate_prefix(&payload, NON_JSON_BODY_PREFIX_CHARS),
            });
        }

        Ok(payload)
    }

    fn wait_for_rate_limit(&self) {
        let Some(limiter) = &self.rate_limiter else {
            return;
        };
        let delay = reserve_rate_limit_delay(limiter);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

#[cfg(any(feature = "blocking", feature = "async"))]
fn lock_rate_limiter(limiter: &Mutex<RateLimiter>) -> std::sync::MutexGuard<'_, RateLimiter> {
    match limiter.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(any(feature = "blocking", feature = "async"))]
fn reserve_rate_limit_delay(limiter: &Mutex<RateLimiter>) -> Duration {
    let mut limiter = lock_rate_limiter(limiter);
    limiter.reserve_delay()
}

#[cfg(feature = "async")]
impl AsyncMoexClient {
    /// Создать builder для конфигурации асинхронного клиента ISS.
    pub fn builder() -> AsyncMoexClientBuilder {
        AsyncMoexClientBuilder {
            base_url: None,
            metadata: false,
            client: None,
            http_client: reqwest::Client::builder(),
            rate_limit: None,
            rate_limit_sleep: None,
        }
    }

    /// Создать асинхронный клиент с базовым URL ISS по умолчанию (`iss.meta=off`).
    pub fn new() -> Result<Self, MoexError> {
        Self::builder().build()
    }

    /// Создать асинхронный клиент с базовым URL ISS по умолчанию и `iss.meta=on`.
    pub fn new_with_metadata() -> Result<Self, MoexError> {
        Self::builder().metadata(true).build()
    }

    /// Создать асинхронный клиент на базе переданного `reqwest`-клиента (`iss.meta=off`).
    pub fn with_client(client: reqwest::Client) -> Result<Self, MoexError> {
        Self::builder().client(client).build()
    }

    /// Создать асинхронный клиент на базе переданного `reqwest`-клиента и включить `iss.meta`.
    pub fn with_client_with_metadata(client: reqwest::Client) -> Result<Self, MoexError> {
        Self::builder().metadata(true).client(client).build()
    }

    /// Создать асинхронный клиент с явным базовым URL и готовым HTTP-клиентом (`iss.meta=off`).
    pub fn with_base_url(client: reqwest::Client, base_url: Url) -> Self {
        Self::with_base_url_and_rate_limit(client, base_url, false, None)
    }

    /// Создать асинхронный клиент с явным базовым URL, HTTP-клиентом и `iss.meta=on`.
    pub fn with_base_url_with_metadata(client: reqwest::Client, base_url: Url) -> Self {
        Self::with_base_url_and_rate_limit(client, base_url, true, None)
    }

    /// Текущее ограничение частоты запросов, если оно включено.
    pub fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
            .as_ref()
            .map(|rate_limit| lock_rate_limiter(&rate_limit.limiter).limit())
    }

    fn with_base_url_and_rate_limit(
        client: reqwest::Client,
        base_url: Url,
        metadata: bool,
        rate_limit: Option<AsyncRateLimitState>,
    ) -> Self {
        Self {
            base_url,
            metadata,
            client,
            rate_limit,
        }
    }

    /// Создать асинхронный raw-builder для произвольного ISS endpoint.
    pub fn raw(&self) -> AsyncRawIssRequestBuilder<'_> {
        AsyncRawIssRequestBuilder {
            client: self,
            path: None,
            query: Vec::new(),
        }
    }

    /// Создать асинхронный raw-builder для типизированного ISS endpoint-а.
    ///
    /// Builder автоматически получает `path` и значение `iss.only` по умолчанию.
    pub fn raw_endpoint(&self, endpoint: IssEndpoint<'_>) -> AsyncRawIssRequestBuilder<'_> {
        let path = endpoint.path();
        let request = self.raw().path(path);
        match endpoint.default_table() {
            Some(table) => request.only(table),
            None => request,
        }
    }

    /// Получить список индексов из таблицы `indices`.
    pub async fn indexes(&self) -> Result<Vec<Index>, MoexError> {
        let payload = self
            .get_payload(
                INDEXES_ENDPOINT,
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "indices"),
                    (INDICES_COLUMNS_PARAM, INDICES_COLUMNS),
                ],
            )
            .await?;
        decode_indexes_json_payload(&payload)
    }

    /// Получить состав индекса (`analytics`) с единым режимом выборки страниц.
    pub async fn index_analytics_query(
        &self,
        indexid: &IndexId,
        page_request: PageRequest,
    ) -> Result<Vec<IndexAnalytics>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_index_analytics_page(indexid, Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_index_analytics_page(indexid, pagination).await
            }
            PageRequest::All { page_limit } => {
                self.index_analytics_pages(indexid, page_limit).all().await
            }
        }
    }

    /// Создать асинхронный ленивый paginator страниц `index_analytics`.
    pub fn index_analytics_pages<'a>(
        &'a self,
        indexid: &'a IndexId,
        page_limit: NonZeroU32,
    ) -> AsyncIndexAnalyticsPages<'a> {
        AsyncIndexAnalyticsPages {
            client: self,
            indexid,
            pagination: PaginationTracker::new(
                index_analytics_endpoint(indexid),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить обороты ISS (`/iss/turnovers`).
    pub async fn turnovers(&self) -> Result<Vec<Turnover>, MoexError> {
        let payload = self
            .get_payload(
                TURNOVERS_ENDPOINT,
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "turnovers"),
                    (TURNOVERS_COLUMNS_PARAM, TURNOVERS_COLUMNS),
                ],
            )
            .await?;
        decode_turnovers_json_with_endpoint(&payload, TURNOVERS_ENDPOINT)
    }

    /// Получить обороты ISS по движку (`/iss/engines/{engine}/turnovers`).
    pub async fn engine_turnovers(&self, engine: &EngineName) -> Result<Vec<Turnover>, MoexError> {
        let endpoint = engine_turnovers_endpoint(engine);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "turnovers"),
                    (TURNOVERS_COLUMNS_PARAM, TURNOVERS_COLUMNS),
                ],
            )
            .await?;
        decode_turnovers_json_with_endpoint(&payload, endpoint.as_str())
    }

    #[cfg(feature = "news")]
    /// Получить новости ISS (`sitenews`) с единым режимом выборки страниц.
    pub async fn sitenews_query(
        &self,
        page_request: PageRequest,
    ) -> Result<Vec<SiteNews>, MoexError> {
        match page_request {
            PageRequest::FirstPage => self.fetch_sitenews_page(Pagination::default()).await,
            PageRequest::Page(pagination) => self.fetch_sitenews_page(pagination).await,
            PageRequest::All { page_limit } => self.sitenews_pages(page_limit).all().await,
        }
    }

    #[cfg(feature = "news")]
    /// Создать асинхронный ленивый paginator страниц `sitenews`.
    pub fn sitenews_pages<'a>(&'a self, page_limit: NonZeroU32) -> AsyncSiteNewsPages<'a> {
        AsyncSiteNewsPages {
            client: self,
            pagination: PaginationTracker::new(
                SITENEWS_ENDPOINT,
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    #[cfg(feature = "news")]
    /// Получить события ISS (`events`) с единым режимом выборки страниц.
    pub async fn events_query(&self, page_request: PageRequest) -> Result<Vec<Event>, MoexError> {
        match page_request {
            PageRequest::FirstPage => self.fetch_events_page(Pagination::default()).await,
            PageRequest::Page(pagination) => self.fetch_events_page(pagination).await,
            PageRequest::All { page_limit } => self.events_pages(page_limit).all().await,
        }
    }

    #[cfg(feature = "news")]
    /// Создать асинхронный ленивый paginator страниц `events`.
    pub fn events_pages<'a>(&'a self, page_limit: NonZeroU32) -> AsyncEventsPages<'a> {
        AsyncEventsPages {
            client: self,
            pagination: PaginationTracker::new(
                EVENTS_ENDPOINT,
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить `secstats` с единым режимом выборки страниц.
    pub async fn secstats_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        page_request: PageRequest,
    ) -> Result<Vec<SecStat>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_secstats_page(engine, market, Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_secstats_page(engine, market, pagination).await
            }
            PageRequest::All { page_limit } => {
                self.secstats_pages(engine, market, page_limit).all().await
            }
        }
    }

    /// Создать асинхронный ленивый paginator страниц `secstats`.
    pub fn secstats_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        page_limit: NonZeroU32,
    ) -> AsyncSecStatsPages<'a> {
        AsyncSecStatsPages {
            client: self,
            engine,
            market,
            pagination: PaginationTracker::new(
                secstats_endpoint(engine, market),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить доступные торговые движки ISS (`engines`).
    pub async fn engines(&self) -> Result<Vec<Engine>, MoexError> {
        let payload = self
            .get_payload(
                ENGINES_ENDPOINT,
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "engines"),
                    (ENGINES_COLUMNS_PARAM, ENGINES_COLUMNS),
                ],
            )
            .await?;
        decode_engines_json_payload(&payload)
    }

    /// Получить рынки (`markets`) для заданного движка.
    pub async fn markets(&self, engine: &EngineName) -> Result<Vec<Market>, MoexError> {
        let endpoint = markets_endpoint(engine);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "markets"),
                    (MARKETS_COLUMNS_PARAM, MARKETS_COLUMNS),
                ],
            )
            .await?;
        decode_markets_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить режимы торгов (`boards`) для пары движок/рынок.
    pub async fn boards(
        &self,
        engine: &EngineName,
        market: &MarketName,
    ) -> Result<Vec<Board>, MoexError> {
        let endpoint = boards_endpoint(engine, market);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "boards"),
                    (BOARDS_COLUMNS_PARAM, BOARDS_COLUMNS),
                ],
            )
            .await?;
        decode_boards_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить режимы торгов инструмента (`boards`) из endpoint `securities/{secid}`.
    pub async fn security_boards(&self, security: &SecId) -> Result<Vec<SecurityBoard>, MoexError> {
        let endpoint = security_boards_endpoint(security);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "boards"),
                    (BOARDS_COLUMNS_PARAM, SECURITY_BOARDS_COLUMNS),
                ],
            )
            .await?;
        decode_security_boards_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить карточку инструмента (`securities`) из endpoint `securities/{secid}`.
    ///
    /// Возвращает `Ok(None)`, если таблица `securities` пустая.
    pub async fn security_info(&self, security: &SecId) -> Result<Option<Security>, MoexError> {
        let endpoint = security_endpoint(security);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "securities"),
                    (SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS),
                ],
            )
            .await?;
        let securities = decode_securities_json_with_endpoint(&payload, endpoint.as_str())?;
        optional_single_security(endpoint.as_str(), securities)
    }

    #[cfg(feature = "history")]
    /// Получить диапазон доступных исторических дат по инструменту и board.
    ///
    /// Возвращает `Ok(None)`, если таблица `dates` пустая.
    pub async fn history_dates(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
    ) -> Result<Option<HistoryDates>, MoexError> {
        let endpoint = history_dates_endpoint(engine, market, board, security);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "dates"),
                ],
            )
            .await?;
        let dates = decode_history_dates_json_with_endpoint(&payload, endpoint.as_str())?;
        optional_single_history_dates(endpoint.as_str(), dates)
    }

    #[cfg(feature = "history")]
    /// Получить исторические данные (`history`) с единым режимом выборки страниц.
    pub async fn history_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        page_request: PageRequest,
    ) -> Result<Vec<HistoryRecord>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_history_page(engine, market, board, security, Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_history_page(engine, market, board, security, pagination)
                    .await
            }
            PageRequest::All { page_limit } => {
                self.history_pages(engine, market, board, security, page_limit)
                    .all()
                    .await
            }
        }
    }

    #[cfg(feature = "history")]
    /// Создать асинхронный ленивый paginator страниц `history`.
    pub fn history_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
        page_limit: NonZeroU32,
    ) -> AsyncHistoryPages<'a> {
        AsyncHistoryPages {
            client: self,
            engine,
            market,
            board,
            security,
            pagination: PaginationTracker::new(
                history_endpoint(engine, market, board, security),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить снимки инструментов (`LOTSIZE` и `LAST`) для режима торгов.
    pub async fn board_snapshots(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
    ) -> Result<Vec<SecuritySnapshot>, MoexError> {
        let endpoint = securities_endpoint(engine, market, board);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "securities,marketdata"),
                    (SECURITIES_COLUMNS_PARAM, SECURITIES_SNAPSHOT_COLUMNS),
                    (MARKETDATA_COLUMNS_PARAM, MARKETDATA_LAST_COLUMNS),
                ],
            )
            .await?;
        decode_board_security_snapshots_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить снимки инструментов (`LOTSIZE` и `LAST`) по данным `SecurityBoard`.
    pub async fn board_security_snapshots(
        &self,
        board: &SecurityBoard,
    ) -> Result<Vec<SecuritySnapshot>, MoexError> {
        self.board_snapshots(board.engine(), board.market(), board.boardid())
            .await
    }

    /// Получить глобальный список инструментов (`/iss/securities`) с единым режимом выборки страниц.
    pub async fn global_securities_query(
        &self,
        page_request: PageRequest,
    ) -> Result<Vec<Security>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_global_securities_page(Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => self.fetch_global_securities_page(pagination).await,
            PageRequest::All { page_limit } => self.global_securities_pages(page_limit).all().await,
        }
    }

    /// Создать асинхронный ленивый paginator страниц глобального `securities`.
    pub fn global_securities_pages<'a>(
        &'a self,
        page_limit: NonZeroU32,
    ) -> AsyncGlobalSecuritiesPages<'a> {
        AsyncGlobalSecuritiesPages {
            client: self,
            pagination: PaginationTracker::new(
                GLOBAL_SECURITIES_ENDPOINT,
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить карточку инструмента на уровне рынка (`.../markets/{market}/securities/{secid}`).
    ///
    /// Возвращает `Ok(None)`, если endpoint не содержит строк `securities`.
    pub async fn market_security_info(
        &self,
        engine: &EngineName,
        market: &MarketName,
        security: &SecId,
    ) -> Result<Option<Security>, MoexError> {
        let endpoint = market_security_endpoint(engine, market, security);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "securities"),
                    (SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS),
                ],
            )
            .await?;
        let securities = decode_securities_json_with_endpoint(&payload, endpoint.as_str())?;
        optional_single_security(endpoint.as_str(), securities)
    }

    /// Получить инструменты (`securities`) на уровне рынка с единым режимом выборки страниц.
    pub async fn market_securities_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        page_request: PageRequest,
    ) -> Result<Vec<Security>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_market_securities_page(engine, market, Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_market_securities_page(engine, market, pagination)
                    .await
            }
            PageRequest::All { page_limit } => {
                self.market_securities_pages(engine, market, page_limit)
                    .all()
                    .await
            }
        }
    }

    /// Создать асинхронный ленивый paginator страниц market-level `securities`.
    pub fn market_securities_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        page_limit: NonZeroU32,
    ) -> AsyncMarketSecuritiesPages<'a> {
        AsyncMarketSecuritiesPages {
            client: self,
            engine,
            market,
            pagination: PaginationTracker::new(
                market_securities_endpoint(engine, market),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить market-level стакан (`orderbook`) по первой странице ISS.
    pub async fn market_orderbook(
        &self,
        engine: &EngineName,
        market: &MarketName,
    ) -> Result<Vec<OrderbookLevel>, MoexError> {
        let endpoint = market_orderbook_endpoint(engine, market);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "orderbook"),
                    (ORDERBOOK_COLUMNS_PARAM, ORDERBOOK_COLUMNS),
                ],
            )
            .await?;
        decode_orderbook_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить доступные границы свечей (`candleborders`) по инструменту.
    pub async fn candle_borders(
        &self,
        engine: &EngineName,
        market: &MarketName,
        security: &SecId,
    ) -> Result<Vec<CandleBorder>, MoexError> {
        let endpoint = candleborders_endpoint(engine, market, security);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[(ISS_META_PARAM, metadata_value(self.metadata))],
            )
            .await?;
        decode_candle_borders_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить market-level сделки (`trades`) с единым режимом выборки страниц.
    pub async fn market_trades_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        page_request: PageRequest,
    ) -> Result<Vec<Trade>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_market_trades_page(engine, market, Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_market_trades_page(engine, market, pagination)
                    .await
            }
            PageRequest::All { page_limit } => {
                self.market_trades_pages(engine, market, page_limit)
                    .all()
                    .await
            }
        }
    }

    /// Создать асинхронный ленивый paginator страниц market-level `trades`.
    pub fn market_trades_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        page_limit: NonZeroU32,
    ) -> AsyncMarketTradesPages<'a> {
        AsyncMarketTradesPages {
            client: self,
            engine,
            market,
            pagination: PaginationTracker::new(
                market_trades_endpoint(engine, market),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить инструменты (`securities`) с единым режимом выборки страниц.
    pub async fn securities_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        page_request: PageRequest,
    ) -> Result<Vec<Security>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_securities_page(engine, market, board, Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_securities_page(engine, market, board, pagination)
                    .await
            }
            PageRequest::All { page_limit } => {
                self.securities_pages(engine, market, board, page_limit)
                    .all()
                    .await
            }
        }
    }

    /// Создать асинхронный ленивый paginator страниц `securities`.
    pub fn securities_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        page_limit: NonZeroU32,
    ) -> AsyncSecuritiesPages<'a> {
        AsyncSecuritiesPages {
            client: self,
            engine,
            market,
            board,
            pagination: PaginationTracker::new(
                securities_endpoint(engine, market, board),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить текущий стакан (`orderbook`) по инструменту.
    pub async fn orderbook(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
    ) -> Result<Vec<OrderbookLevel>, MoexError> {
        let endpoint = orderbook_endpoint(engine, market, board, security);
        let payload = self
            .get_payload(
                endpoint.as_str(),
                &[
                    (ISS_META_PARAM, metadata_value(self.metadata)),
                    (ISS_ONLY_PARAM, "orderbook"),
                    (ORDERBOOK_COLUMNS_PARAM, ORDERBOOK_COLUMNS),
                ],
            )
            .await?;
        decode_orderbook_json_with_endpoint(&payload, endpoint.as_str())
    }

    /// Получить свечи (`candles`) с единым режимом выборки страниц.
    pub async fn candles_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        query: CandleQuery,
        page_request: PageRequest,
    ) -> Result<Vec<Candle>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_candles_page(
                    engine,
                    market,
                    board,
                    security,
                    query,
                    Pagination::default(),
                )
                .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_candles_page(engine, market, board, security, query, pagination)
                    .await
            }
            PageRequest::All { page_limit } => {
                self.candles_pages(engine, market, board, security, query, page_limit)
                    .all()
                    .await
            }
        }
    }

    /// Создать асинхронный ленивый paginator страниц `candles`.
    pub fn candles_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
        query: CandleQuery,
        page_limit: NonZeroU32,
    ) -> AsyncCandlesPages<'a> {
        AsyncCandlesPages {
            client: self,
            engine,
            market,
            board,
            security,
            query,
            pagination: PaginationTracker::new(
                candles_endpoint(engine, market, board, security),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Получить сделки (`trades`) с единым режимом выборки страниц.
    pub async fn trades_query(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        page_request: PageRequest,
    ) -> Result<Vec<Trade>, MoexError> {
        match page_request {
            PageRequest::FirstPage => {
                self.fetch_trades_page(engine, market, board, security, Pagination::default())
                    .await
            }
            PageRequest::Page(pagination) => {
                self.fetch_trades_page(engine, market, board, security, pagination)
                    .await
            }
            PageRequest::All { page_limit } => {
                self.trades_pages(engine, market, board, security, page_limit)
                    .all()
                    .await
            }
        }
    }

    /// Создать асинхронный ленивый paginator страниц `trades`.
    pub fn trades_pages<'a>(
        &'a self,
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
        page_limit: NonZeroU32,
    ) -> AsyncTradesPages<'a> {
        AsyncTradesPages {
            client: self,
            engine,
            market,
            board,
            security,
            pagination: PaginationTracker::new(
                trades_endpoint(engine, market, board, security),
                page_limit,
                RepeatPagePolicy::Error,
            ),
        }
    }

    /// Зафиксировать async owning-scope по `engine` из ergonomic-входа.
    pub fn engine<E>(&self, engine: E) -> Result<AsyncOwnedEngineScope<'_>, ParseEngineNameError>
    where
        E: TryInto<EngineName>,
        E::Error: Into<ParseEngineNameError>,
    {
        let engine = engine.try_into().map_err(Into::into)?;
        Ok(AsyncOwnedEngineScope {
            client: self,
            engine,
        })
    }

    /// Shortcut для часто используемого engine `stock`.
    pub fn stock(&self) -> Result<AsyncOwnedEngineScope<'_>, ParseEngineNameError> {
        self.engine("stock")
    }

    /// Зафиксировать async owning-scope по `indexid` из ergonomic-входа.
    pub fn index<I>(&self, indexid: I) -> Result<AsyncOwnedIndexScope<'_>, ParseIndexError>
    where
        I: TryInto<IndexId>,
        I::Error: Into<ParseIndexError>,
    {
        let indexid = indexid.try_into().map_err(Into::into)?;
        Ok(AsyncOwnedIndexScope {
            client: self,
            indexid,
        })
    }

    /// Зафиксировать async owning-scope по `secid` из ergonomic-входа.
    pub fn security<S>(
        &self,
        security: S,
    ) -> Result<AsyncOwnedSecurityResourceScope<'_>, ParseSecIdError>
    where
        S: TryInto<SecId>,
        S::Error: Into<ParseSecIdError>,
    {
        let security = security.try_into().map_err(Into::into)?;
        Ok(AsyncOwnedSecurityResourceScope {
            client: self,
            security,
        })
    }

    async fn fetch_index_analytics_page(
        &self,
        indexid: &IndexId,
        pagination: Pagination,
    ) -> Result<Vec<IndexAnalytics>, MoexError> {
        let endpoint = index_analytics_endpoint(indexid);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "analytics")
                .append_pair(ANALYTICS_COLUMNS_PARAM, ANALYTICS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_index_analytics_json_with_endpoint(&payload, endpoint.as_str())
    }

    async fn fetch_securities_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        pagination: Pagination,
    ) -> Result<Vec<Security>, MoexError> {
        let endpoint = securities_endpoint(engine, market, board);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "securities")
                .append_pair(SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_securities_json_with_endpoint(&payload, endpoint.as_str())
    }

    async fn fetch_global_securities_page(
        &self,
        pagination: Pagination,
    ) -> Result<Vec<Security>, MoexError> {
        let endpoint = GLOBAL_SECURITIES_ENDPOINT;
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "securities")
                .append_pair(SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint, endpoint_url).await?;
        decode_securities_json_with_endpoint(&payload, endpoint)
    }

    #[cfg(feature = "news")]
    async fn fetch_sitenews_page(
        &self,
        pagination: Pagination,
    ) -> Result<Vec<SiteNews>, MoexError> {
        let endpoint = SITENEWS_ENDPOINT;
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "sitenews")
                .append_pair(SITENEWS_COLUMNS_PARAM, SITENEWS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint, endpoint_url).await?;
        decode_sitenews_json_with_endpoint(&payload, endpoint)
    }

    #[cfg(feature = "news")]
    async fn fetch_events_page(&self, pagination: Pagination) -> Result<Vec<Event>, MoexError> {
        let endpoint = EVENTS_ENDPOINT;
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "events")
                .append_pair(EVENTS_COLUMNS_PARAM, EVENTS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint, endpoint_url).await?;
        decode_events_json_with_endpoint(&payload, endpoint)
    }

    async fn fetch_market_securities_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        pagination: Pagination,
    ) -> Result<Vec<Security>, MoexError> {
        let endpoint = market_securities_endpoint(engine, market);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "securities")
                .append_pair(SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_securities_json_with_endpoint(&payload, endpoint.as_str())
    }

    async fn fetch_market_trades_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        pagination: Pagination,
    ) -> Result<Vec<Trade>, MoexError> {
        let endpoint = market_trades_endpoint(engine, market);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "trades")
                .append_pair(TRADES_COLUMNS_PARAM, TRADES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_trades_json_with_endpoint(&payload, endpoint.as_str())
    }

    async fn fetch_secstats_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        pagination: Pagination,
    ) -> Result<Vec<SecStat>, MoexError> {
        let endpoint = secstats_endpoint(engine, market);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "secstats")
                .append_pair(SECSTATS_COLUMNS_PARAM, SECSTATS_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_secstats_json_with_endpoint(&payload, endpoint.as_str())
    }

    async fn fetch_candles_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        query: CandleQuery,
        pagination: Pagination,
    ) -> Result<Vec<Candle>, MoexError> {
        let endpoint = candles_endpoint(engine, market, board, security);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query_pairs = endpoint_url.query_pairs_mut();
            query_pairs
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "candles")
                .append_pair(CANDLES_COLUMNS_PARAM, CANDLES_COLUMNS);
        }
        append_candle_query_to_url(&mut endpoint_url, query);
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_candles_json_with_endpoint(&payload, endpoint.as_str())
    }

    async fn fetch_trades_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        pagination: Pagination,
    ) -> Result<Vec<Trade>, MoexError> {
        let endpoint = trades_endpoint(engine, market, board, security);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "trades")
                .append_pair(TRADES_COLUMNS_PARAM, TRADES_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_trades_json_with_endpoint(&payload, endpoint.as_str())
    }

    #[cfg(feature = "history")]
    async fn fetch_history_page(
        &self,
        engine: &EngineName,
        market: &MarketName,
        board: &BoardId,
        security: &SecId,
        pagination: Pagination,
    ) -> Result<Vec<HistoryRecord>, MoexError> {
        let endpoint = history_endpoint(engine, market, board, security);
        let mut endpoint_url = self.endpoint_url(endpoint.as_str())?;
        {
            let mut query = endpoint_url.query_pairs_mut();
            query
                .append_pair(ISS_META_PARAM, metadata_value(self.metadata))
                .append_pair(ISS_ONLY_PARAM, "history")
                .append_pair(HISTORY_COLUMNS_PARAM, HISTORY_COLUMNS);
        }
        append_pagination_to_url(&mut endpoint_url, pagination);

        let payload = self.fetch_payload(endpoint.as_str(), endpoint_url).await?;
        decode_history_json_with_endpoint(&payload, endpoint.as_str())
    }

    fn endpoint_url(&self, endpoint: &str) -> Result<Url, MoexError> {
        self.base_url
            .join(endpoint)
            .map_err(|source| MoexError::EndpointUrl {
                endpoint: endpoint.to_owned().into_boxed_str(),
                reason: source.to_string(),
            })
    }

    async fn get_payload(
        &self,
        endpoint: &str,
        query_params: &[(&'static str, &'static str)],
    ) -> Result<String, MoexError> {
        let mut endpoint_url = self.endpoint_url(endpoint)?;
        {
            let mut url_query = endpoint_url.query_pairs_mut();
            for (key, value) in query_params {
                url_query.append_pair(key, value);
            }
        }
        self.fetch_payload(endpoint, endpoint_url).await
    }

    async fn fetch_payload(&self, endpoint: &str, endpoint_url: Url) -> Result<String, MoexError> {
        self.wait_for_rate_limit().await;
        let response = self
            .client
            .get(endpoint_url)
            .send()
            .await
            .map_err(|source| MoexError::Request {
                endpoint: endpoint.to_owned().into_boxed_str(),
                source,
            })?;
        let status = response.status();

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_owned().into_boxed_str());

        let payload = response
            .text()
            .await
            .map_err(|source| MoexError::ReadBody {
                endpoint: endpoint.to_owned().into_boxed_str(),
                source,
            })?;

        if !status.is_success() {
            return Err(MoexError::HttpStatus {
                endpoint: endpoint.to_owned().into_boxed_str(),
                status,
                content_type,
                body_prefix: truncate_prefix(&payload, NON_JSON_BODY_PREFIX_CHARS),
            });
        }

        if !looks_like_json_payload(content_type.as_deref(), &payload) {
            return Err(MoexError::NonJsonPayload {
                endpoint: endpoint.to_owned().into_boxed_str(),
                content_type,
                body_prefix: truncate_prefix(&payload, NON_JSON_BODY_PREFIX_CHARS),
            });
        }

        Ok(payload)
    }

    async fn wait_for_rate_limit(&self) {
        let Some(rate_limit) = &self.rate_limit else {
            return;
        };
        let delay = reserve_rate_limit_delay(&rate_limit.limiter);
        if !delay.is_zero() {
            (rate_limit.sleep)(delay).await;
        }
    }
}

#[cfg(feature = "blocking")]
impl<'a> RawIssRequestBuilder<'a> {
    /// Установить endpoint-path относительно `/iss/`.
    ///
    /// Допускаются формы:
    /// - `engines`
    /// - `engines.json`
    /// - `/iss/engines`
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into().into_boxed_str());
        self
    }

    /// Добавить query-параметр.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query
            .push((key.into().into_boxed_str(), value.into().into_boxed_str()));
        self
    }

    /// Добавить параметр `iss.only`.
    pub fn only(self, tables: impl Into<String>) -> Self {
        self.param(ISS_ONLY_PARAM, tables)
    }

    /// Добавить параметр `<table>.columns`.
    pub fn columns(self, table: impl Into<String>, columns: impl Into<String>) -> Self {
        let mut key = table.into();
        key.push_str(".columns");
        self.param(key, columns)
    }

    /// Явно задать `iss.meta` для текущего raw-запроса.
    pub fn metadata(self, metadata: IssToggle) -> Self {
        self.param(ISS_META_PARAM, metadata.as_query_value())
    }

    /// Добавить параметр `iss.data`.
    pub fn data(self, data: IssToggle) -> Self {
        self.param(ISS_DATA_PARAM, data.as_query_value())
    }

    /// Добавить параметр `iss.json`.
    pub fn json(self, json: impl Into<String>) -> Self {
        self.param(ISS_JSON_PARAM, json)
    }

    /// Добавить параметр `iss.version`.
    pub fn version(self, version: IssToggle) -> Self {
        self.param(ISS_VERSION_PARAM, version.as_query_value())
    }

    /// Применить пакет системных `iss.*`-опций.
    pub fn options(mut self, options: IssRequestOptions) -> Self {
        apply_iss_request_options(&mut self.query, options);
        self
    }

    /// Выполнить raw-запрос и вернуть полный HTTP-ответ.
    ///
    /// В отличие от `send_payload`, метод не проверяет `2xx` и JSON-формат.
    pub fn send_response(self) -> Result<RawIssResponse, MoexError> {
        let (_, response) = self.execute_response()?;
        Ok(response)
    }

    /// Выполнить raw-запрос и вернуть тело ответа как строку.
    pub fn send_payload(self) -> Result<String, MoexError> {
        let (_, payload) = self.execute()?;
        Ok(payload)
    }

    /// Выполнить raw-запрос и декодировать JSON в пользовательский тип.
    pub fn send_json<T>(self) -> Result<T, MoexError>
    where
        T: serde::de::DeserializeOwned,
    {
        let (endpoint, payload) = self.execute()?;
        serde_json::from_str(&payload).map_err(|source| MoexError::Decode { endpoint, source })
    }

    /// Выполнить raw-запрос и декодировать строки выбранной ISS-таблицы в пользовательский тип.
    pub fn send_table<T>(self, table: impl Into<String>) -> Result<Vec<T>, MoexError>
    where
        T: serde::de::DeserializeOwned,
    {
        let table = table.into();
        let (endpoint, payload) = self.execute()?;
        decode_raw_table_rows_json_with_endpoint(&payload, endpoint.as_ref(), table.as_str())
    }

    fn execute(self) -> Result<(Box<str>, String), MoexError> {
        let (endpoint, endpoint_url) = self.build_request()?;
        let payload = self.client.fetch_payload(&endpoint, endpoint_url)?;
        Ok((endpoint, payload))
    }

    fn execute_response(self) -> Result<(Box<str>, RawIssResponse), MoexError> {
        let (endpoint, endpoint_url) = self.build_request()?;
        self.client.wait_for_rate_limit();
        let response = self
            .client
            .client
            .get(endpoint_url)
            .send()
            .map_err(|source| MoexError::Request {
                endpoint: endpoint.clone(),
                source,
            })?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().map_err(|source| MoexError::ReadBody {
            endpoint: endpoint.clone(),
            source,
        })?;
        Ok((endpoint, RawIssResponse::new(status, headers, body)))
    }

    fn build_request(&self) -> Result<(Box<str>, Url), MoexError> {
        let endpoint = normalize_raw_endpoint_path(self.path.as_deref())?;
        let mut endpoint_url = self.client.endpoint_url(&endpoint)?;
        let has_meta = self
            .query
            .iter()
            .any(|(key, _)| key.as_ref() == ISS_META_PARAM);
        {
            let mut url_query = endpoint_url.query_pairs_mut();
            if !has_meta {
                url_query.append_pair(ISS_META_PARAM, metadata_value(self.client.metadata));
            }
            for (key, value) in &self.query {
                url_query.append_pair(key, value);
            }
        }
        Ok((endpoint, endpoint_url))
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncRawIssRequestBuilder<'a> {
    /// Установить endpoint-path относительно `/iss/`.
    ///
    /// Допускаются формы:
    /// - `engines`
    /// - `engines.json`
    /// - `/iss/engines`
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into().into_boxed_str());
        self
    }

    /// Добавить query-параметр.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query
            .push((key.into().into_boxed_str(), value.into().into_boxed_str()));
        self
    }

    /// Добавить параметр `iss.only`.
    pub fn only(self, tables: impl Into<String>) -> Self {
        self.param(ISS_ONLY_PARAM, tables)
    }

    /// Добавить параметр `<table>.columns`.
    pub fn columns(self, table: impl Into<String>, columns: impl Into<String>) -> Self {
        let mut key = table.into();
        key.push_str(".columns");
        self.param(key, columns)
    }

    /// Явно задать `iss.meta` для текущего raw-запроса.
    pub fn metadata(self, metadata: IssToggle) -> Self {
        self.param(ISS_META_PARAM, metadata.as_query_value())
    }

    /// Добавить параметр `iss.data`.
    pub fn data(self, data: IssToggle) -> Self {
        self.param(ISS_DATA_PARAM, data.as_query_value())
    }

    /// Добавить параметр `iss.json`.
    pub fn json(self, json: impl Into<String>) -> Self {
        self.param(ISS_JSON_PARAM, json)
    }

    /// Добавить параметр `iss.version`.
    pub fn version(self, version: IssToggle) -> Self {
        self.param(ISS_VERSION_PARAM, version.as_query_value())
    }

    /// Применить пакет системных `iss.*`-опций.
    pub fn options(mut self, options: IssRequestOptions) -> Self {
        apply_iss_request_options(&mut self.query, options);
        self
    }

    /// Выполнить raw-запрос и вернуть полный HTTP-ответ.
    ///
    /// В отличие от `send_payload`, метод не проверяет `2xx` и JSON-формат.
    pub async fn send_response(self) -> Result<RawIssResponse, MoexError> {
        let (_, response) = self.execute_response().await?;
        Ok(response)
    }

    /// Выполнить raw-запрос и вернуть тело ответа как строку.
    pub async fn send_payload(self) -> Result<String, MoexError> {
        let (_, payload) = self.execute().await?;
        Ok(payload)
    }

    /// Выполнить raw-запрос и декодировать JSON в пользовательский тип.
    pub async fn send_json<T>(self) -> Result<T, MoexError>
    where
        T: serde::de::DeserializeOwned,
    {
        let (endpoint, payload) = self.execute().await?;
        serde_json::from_str(&payload).map_err(|source| MoexError::Decode { endpoint, source })
    }

    /// Выполнить raw-запрос и декодировать строки выбранной ISS-таблицы в пользовательский тип.
    pub async fn send_table<T>(self, table: impl Into<String>) -> Result<Vec<T>, MoexError>
    where
        T: serde::de::DeserializeOwned,
    {
        let table = table.into();
        let (endpoint, payload) = self.execute().await?;
        decode_raw_table_rows_json_with_endpoint(&payload, endpoint.as_ref(), table.as_str())
    }

    async fn execute(self) -> Result<(Box<str>, String), MoexError> {
        let (endpoint, endpoint_url) = self.build_request()?;
        let payload = self.client.fetch_payload(&endpoint, endpoint_url).await?;
        Ok((endpoint, payload))
    }

    async fn execute_response(self) -> Result<(Box<str>, RawIssResponse), MoexError> {
        let (endpoint, endpoint_url) = self.build_request()?;
        self.client.wait_for_rate_limit().await;
        let response = self
            .client
            .client
            .get(endpoint_url)
            .send()
            .await
            .map_err(|source| MoexError::Request {
                endpoint: endpoint.clone(),
                source,
            })?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .text()
            .await
            .map_err(|source| MoexError::ReadBody {
                endpoint: endpoint.clone(),
                source,
            })?;
        Ok((endpoint, RawIssResponse::new(status, headers, body)))
    }

    fn build_request(&self) -> Result<(Box<str>, Url), MoexError> {
        let endpoint = normalize_raw_endpoint_path(self.path.as_deref())?;
        let mut endpoint_url = self.client.endpoint_url(&endpoint)?;
        let has_meta = self
            .query
            .iter()
            .any(|(key, _)| key.as_ref() == ISS_META_PARAM);
        {
            let mut url_query = endpoint_url.query_pairs_mut();
            if !has_meta {
                url_query.append_pair(ISS_META_PARAM, metadata_value(self.client.metadata));
            }
            for (key, value) in &self.query {
                url_query.append_pair(key, value);
            }
        }
        Ok((endpoint, endpoint_url))
    }
}

#[cfg(feature = "blocking")]
fn next_page_blocking<T, K, F, G>(
    pagination: &mut PaginationTracker<K>,
    fetch_page: F,
    first_key_of: G,
) -> Result<Option<Vec<T>>, MoexError>
where
    K: Eq,
    F: FnOnce(Pagination) -> Result<Vec<T>, MoexError>,
    G: Fn(&T) -> K,
{
    let Some(paging) = pagination.next_page_request() else {
        return Ok(None);
    };
    let page = fetch_page(paging)?;
    let first_key_on_page = page.first().map(first_key_of);
    match pagination.advance(page.len(), first_key_on_page)? {
        PaginationAdvance::YieldPage => Ok(Some(page)),
        PaginationAdvance::EndOfPages => Ok(None),
    }
}

#[cfg(feature = "async")]
async fn next_page_async<T, K, F, Fut, G>(
    pagination: &mut PaginationTracker<K>,
    fetch_page: F,
    first_key_of: G,
) -> Result<Option<Vec<T>>, MoexError>
where
    K: Eq,
    F: FnOnce(Pagination) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>, MoexError>>,
    G: Fn(&T) -> K,
{
    let Some(paging) = pagination.next_page_request() else {
        return Ok(None);
    };
    let page = fetch_page(paging).await?;
    let first_key_on_page = page.first().map(first_key_of);
    match pagination.advance(page.len(), first_key_on_page)? {
        PaginationAdvance::YieldPage => Ok(Some(page)),
        PaginationAdvance::EndOfPages => Ok(None),
    }
}

#[cfg(feature = "blocking")]
fn collect_pages_blocking<T, F>(mut next_page: F) -> Result<Vec<T>, MoexError>
where
    F: FnMut() -> Result<Option<Vec<T>>, MoexError>,
{
    let mut items = Vec::new();
    while let Some(page) = next_page()? {
        items.extend(page);
    }
    Ok(items)
}

#[cfg(feature = "async")]
impl<'a> AsyncIndexAnalyticsPages<'a> {
    /// Получить следующую страницу `index_analytics`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<IndexAnalytics>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_index_analytics_page(self.indexid, pagination)
            },
            |item| (item.trade_session_date(), item.secid().clone()),
        )
        .await
    }

    /// Собрать все страницы `index_analytics` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<IndexAnalytics>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<IndexAnalytics>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncSecuritiesPages<'a> {
    /// Получить следующую страницу `securities`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<Security>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_securities_page(self.engine, self.market, self.board, pagination)
            },
            |item| item.secid().clone(),
        )
        .await
    }

    /// Собрать все страницы `securities` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<Security>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<Security>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncGlobalSecuritiesPages<'a> {
    /// Получить следующую страницу глобального `securities`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<Security>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| self.client.fetch_global_securities_page(pagination),
            |item| item.secid().clone(),
        )
        .await
    }

    /// Собрать все страницы глобального `securities` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<Security>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<Security>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(all(feature = "async", feature = "news"))]
impl<'a> AsyncSiteNewsPages<'a> {
    /// Получить следующую страницу `sitenews`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<SiteNews>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| self.client.fetch_sitenews_page(pagination),
            SiteNews::id,
        )
        .await
    }

    /// Собрать все страницы `sitenews` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<SiteNews>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<SiteNews>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(all(feature = "async", feature = "news"))]
impl<'a> AsyncEventsPages<'a> {
    /// Получить следующую страницу `events`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<Event>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| self.client.fetch_events_page(pagination),
            Event::id,
        )
        .await
    }

    /// Собрать все страницы `events` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<Event>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<Event>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncMarketSecuritiesPages<'a> {
    /// Получить следующую страницу market-level `securities`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<Security>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_market_securities_page(self.engine, self.market, pagination)
            },
            |item| item.secid().clone(),
        )
        .await
    }

    /// Собрать все страницы market-level `securities` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<Security>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<Security>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncMarketTradesPages<'a> {
    /// Получить следующую страницу market-level `trades`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<Trade>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_market_trades_page(self.engine, self.market, pagination)
            },
            Trade::tradeno,
        )
        .await
    }

    /// Собрать все страницы market-level `trades` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<Trade>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<Trade>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncTradesPages<'a> {
    /// Получить следующую страницу `trades`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<Trade>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client.fetch_trades_page(
                    self.engine,
                    self.market,
                    self.board,
                    self.security,
                    pagination,
                )
            },
            Trade::tradeno,
        )
        .await
    }

    /// Собрать все страницы `trades` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<Trade>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<Trade>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(all(feature = "async", feature = "history"))]
impl<'a> AsyncHistoryPages<'a> {
    /// Получить следующую страницу `history`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<HistoryRecord>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client.fetch_history_page(
                    self.engine,
                    self.market,
                    self.board,
                    self.security,
                    pagination,
                )
            },
            HistoryRecord::tradedate,
        )
        .await
    }

    /// Собрать все страницы `history` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<HistoryRecord>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<HistoryRecord>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncSecStatsPages<'a> {
    /// Получить следующую страницу `secstats`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<SecStat>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_secstats_page(self.engine, self.market, pagination)
            },
            |item| (item.secid().clone(), item.boardid().clone()),
        )
        .await
    }

    /// Собрать все страницы `secstats` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<SecStat>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<SecStat>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncCandlesPages<'a> {
    /// Получить следующую страницу `candles`.
    pub async fn next_page(&mut self) -> Result<Option<Vec<Candle>>, MoexError> {
        next_page_async(
            &mut self.pagination,
            |pagination| {
                self.client.fetch_candles_page(
                    self.engine,
                    self.market,
                    self.board,
                    self.security,
                    self.query,
                    pagination,
                )
            },
            Candle::begin,
        )
        .await
    }

    /// Собрать все страницы `candles` в один `Vec`.
    pub async fn try_collect(mut self) -> Result<Vec<Candle>, MoexError> {
        {
            let mut items = Vec::new();
            while let Some(page) = self.next_page().await? {
                items.extend(page);
            }
            Ok(items)
        }
    }

    /// Алиас для [`Self::try_collect`].
    pub async fn all(self) -> Result<Vec<Candle>, MoexError> {
        self.try_collect().await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncOwnedIndexScope<'a> {
    /// Идентификатор индекса текущего async owning-scope.
    pub fn indexid(&self) -> &IndexId {
        &self.indexid
    }

    /// Получить состав индекса (`analytics`) по текущему async owning-scope.
    pub async fn analytics(
        &self,
        page_request: PageRequest,
    ) -> Result<Vec<IndexAnalytics>, MoexError> {
        self.client
            .index_analytics_query(&self.indexid, page_request)
            .await
    }

    /// Создать асинхронный ленивый paginator `analytics` для текущего индекса.
    pub fn analytics_pages(&self, page_limit: NonZeroU32) -> AsyncIndexAnalyticsPages<'_> {
        self.client.index_analytics_pages(&self.indexid, page_limit)
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncOwnedEngineScope<'a> {
    /// Имя торгового движка текущего async owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Получить доступные рынки (`markets`) для текущего движка.
    pub async fn markets(&self) -> Result<Vec<Market>, MoexError> {
        self.client.markets(&self.engine).await
    }

    /// Получить обороты (`turnovers`) для текущего движка.
    pub async fn turnovers(&self) -> Result<Vec<Turnover>, MoexError> {
        self.client.engine_turnovers(&self.engine).await
    }

    /// Зафиксировать рынок внутри текущего `engine`.
    pub fn market<M>(self, market: M) -> Result<AsyncOwnedMarketScope<'a>, ParseMarketNameError>
    where
        M: TryInto<MarketName>,
        M::Error: Into<ParseMarketNameError>,
    {
        let market = market.try_into().map_err(Into::into)?;
        Ok(AsyncOwnedMarketScope {
            client: self.client,
            engine: self.engine,
            market,
        })
    }

    /// Shortcut для часто используемого рынка `shares`.
    pub fn shares(self) -> Result<AsyncOwnedMarketScope<'a>, ParseMarketNameError> {
        self.market("shares")
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncOwnedMarketScope<'a> {
    /// Имя торгового движка текущего async owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Имя рынка текущего async owning-scope.
    pub fn market(&self) -> &MarketName {
        &self.market
    }

    /// Получить режимы торгов (`boards`) для текущего рынка.
    pub async fn boards(&self) -> Result<Vec<Board>, MoexError> {
        self.client.boards(&self.engine, &self.market).await
    }

    /// Получить инструменты (`securities`) на уровне текущего рынка.
    pub async fn securities(&self, page_request: PageRequest) -> Result<Vec<Security>, MoexError> {
        self.client
            .market_securities_query(&self.engine, &self.market, page_request)
            .await
    }

    /// Создать асинхронный ленивый paginator market-level `securities`.
    pub fn securities_pages(&self, page_limit: NonZeroU32) -> AsyncMarketSecuritiesPages<'_> {
        self.client
            .market_securities_pages(&self.engine, &self.market, page_limit)
    }

    /// Получить market-level стакан (`orderbook`) для текущего рынка.
    pub async fn orderbook(&self) -> Result<Vec<OrderbookLevel>, MoexError> {
        self.client
            .market_orderbook(&self.engine, &self.market)
            .await
    }

    /// Получить market-level сделки (`trades`) для текущего рынка.
    pub async fn trades(&self, page_request: PageRequest) -> Result<Vec<Trade>, MoexError> {
        self.client
            .market_trades_query(&self.engine, &self.market, page_request)
            .await
    }

    /// Создать асинхронный ленивый paginator market-level `trades`.
    pub fn trades_pages(&self, page_limit: NonZeroU32) -> AsyncMarketTradesPages<'_> {
        self.client
            .market_trades_pages(&self.engine, &self.market, page_limit)
    }

    /// Получить `secstats` для текущего рынка.
    pub async fn secstats(&self, page_request: PageRequest) -> Result<Vec<SecStat>, MoexError> {
        self.client
            .secstats_query(&self.engine, &self.market, page_request)
            .await
    }

    /// Создать асинхронный ленивый paginator `secstats`.
    pub fn secstats_pages(&self, page_limit: NonZeroU32) -> AsyncSecStatsPages<'_> {
        self.client
            .secstats_pages(&self.engine, &self.market, page_limit)
    }

    /// Получить доступные границы свечей (`candleborders`) по инструменту.
    pub async fn candle_borders(&self, security: &SecId) -> Result<Vec<CandleBorder>, MoexError> {
        self.client
            .candle_borders(&self.engine, &self.market, security)
            .await
    }

    /// Зафиксировать инструмент в рамках текущего `engine/market`.
    pub fn security<S>(
        self,
        security: S,
    ) -> Result<AsyncOwnedMarketSecurityScope<'a>, ParseSecIdError>
    where
        S: TryInto<SecId>,
        S::Error: Into<ParseSecIdError>,
    {
        let security = security.try_into().map_err(Into::into)?;
        Ok(AsyncOwnedMarketSecurityScope {
            client: self.client,
            engine: self.engine,
            market: self.market,
            security,
        })
    }

    /// Зафиксировать `board` внутри текущего `engine/market`.
    pub fn board<B>(self, board: B) -> Result<AsyncOwnedBoardScope<'a>, ParseBoardIdError>
    where
        B: TryInto<BoardId>,
        B::Error: Into<ParseBoardIdError>,
    {
        let board = board.try_into().map_err(Into::into)?;
        Ok(AsyncOwnedBoardScope {
            client: self.client,
            engine: self.engine,
            market: self.market,
            board,
        })
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncOwnedBoardScope<'a> {
    /// Имя торгового движка текущего async owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Имя рынка текущего async owning-scope.
    pub fn market(&self) -> &MarketName {
        &self.market
    }

    /// Идентификатор режима торгов текущего async owning-scope.
    pub fn board(&self) -> &BoardId {
        &self.board
    }

    /// Получить инструменты (`securities`) по текущему async owning-scope.
    pub async fn securities(&self, page_request: PageRequest) -> Result<Vec<Security>, MoexError> {
        self.client
            .securities_query(&self.engine, &self.market, &self.board, page_request)
            .await
    }

    /// Создать асинхронный ленивый paginator `securities` по текущему async owning-scope.
    pub fn securities_pages(&self, page_limit: NonZeroU32) -> AsyncSecuritiesPages<'_> {
        self.client
            .securities_pages(&self.engine, &self.market, &self.board, page_limit)
    }

    /// Получить снимки инструментов (`LOTSIZE` и `LAST`) для текущего owning-scope.
    pub async fn snapshots(&self) -> Result<Vec<SecuritySnapshot>, MoexError> {
        self.client
            .board_snapshots(&self.engine, &self.market, &self.board)
            .await
    }

    /// Зафиксировать инструмент в рамках текущего `engine/market/board`.
    pub fn security<S>(self, security: S) -> Result<AsyncOwnedSecurityScope<'a>, ParseSecIdError>
    where
        S: TryInto<SecId>,
        S::Error: Into<ParseSecIdError>,
    {
        let security = security.try_into().map_err(Into::into)?;
        Ok(AsyncOwnedSecurityScope {
            client: self.client,
            engine: self.engine,
            market: self.market,
            board: self.board,
            security,
        })
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncOwnedSecurityResourceScope<'a> {
    /// Идентификатор инструмента текущего async owning-scope.
    pub fn secid(&self) -> &SecId {
        &self.security
    }

    /// Получить карточку текущего инструмента.
    pub async fn info(&self) -> Result<Option<Security>, MoexError> {
        self.client.security_info(&self.security).await
    }

    /// Получить режимы торгов (`boards`) для текущего инструмента.
    pub async fn boards(&self) -> Result<Vec<SecurityBoard>, MoexError> {
        self.client.security_boards(&self.security).await
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncOwnedSecurityScope<'a> {
    /// Идентификатор инструмента текущего async owning-scope.
    pub fn security(&self) -> &SecId {
        &self.security
    }

    /// Получить стакан (`orderbook`) по текущему инструменту.
    pub async fn orderbook(&self) -> Result<Vec<OrderbookLevel>, MoexError> {
        self.client
            .orderbook(&self.engine, &self.market, &self.board, &self.security)
            .await
    }

    #[cfg(feature = "history")]
    /// Получить диапазон доступных исторических дат по текущему инструменту.
    pub async fn history_dates(&self) -> Result<Option<HistoryDates>, MoexError> {
        self.client
            .history_dates(&self.engine, &self.market, &self.board, &self.security)
            .await
    }

    #[cfg(feature = "history")]
    /// Получить исторические данные (`history`) по текущему инструменту.
    pub async fn history(
        &self,
        page_request: PageRequest,
    ) -> Result<Vec<HistoryRecord>, MoexError> {
        self.client
            .history_query(
                &self.engine,
                &self.market,
                &self.board,
                &self.security,
                page_request,
            )
            .await
    }

    #[cfg(feature = "history")]
    /// Создать асинхронный ленивый paginator `history` по текущему инструменту.
    pub fn history_pages(&self, page_limit: NonZeroU32) -> AsyncHistoryPages<'_> {
        self.client.history_pages(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            page_limit,
        )
    }

    /// Получить сделки (`trades`) по текущему инструменту.
    pub async fn trades(&self, page_request: PageRequest) -> Result<Vec<Trade>, MoexError> {
        self.client
            .trades_query(
                &self.engine,
                &self.market,
                &self.board,
                &self.security,
                page_request,
            )
            .await
    }

    /// Создать асинхронный ленивый paginator `trades` по текущему инструменту.
    pub fn trades_pages(&self, page_limit: NonZeroU32) -> AsyncTradesPages<'_> {
        self.client.trades_pages(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            page_limit,
        )
    }

    /// Получить свечи (`candles`) по текущему инструменту.
    pub async fn candles(
        &self,
        query: CandleQuery,
        page_request: PageRequest,
    ) -> Result<Vec<Candle>, MoexError> {
        self.client
            .candles_query(
                &self.engine,
                &self.market,
                &self.board,
                &self.security,
                query,
                page_request,
            )
            .await
    }

    /// Создать асинхронный ленивый paginator `candles` по текущему инструменту.
    pub fn candles_pages(
        &self,
        query: CandleQuery,
        page_limit: NonZeroU32,
    ) -> AsyncCandlesPages<'_> {
        self.client.candles_pages(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            query,
            page_limit,
        )
    }
}

#[cfg(feature = "async")]
impl<'a> AsyncOwnedMarketSecurityScope<'a> {
    /// Имя торгового движка текущего async owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Имя рынка текущего async owning-scope.
    pub fn market(&self) -> &MarketName {
        &self.market
    }

    /// Идентификатор инструмента текущего async owning-scope.
    pub fn security(&self) -> &SecId {
        &self.security
    }

    /// Получить карточку текущего инструмента на уровне рынка.
    pub async fn info(&self) -> Result<Option<Security>, MoexError> {
        self.client
            .market_security_info(&self.engine, &self.market, &self.security)
            .await
    }

    /// Получить доступные границы свечей (`candleborders`) по текущему инструменту.
    pub async fn candle_borders(&self) -> Result<Vec<CandleBorder>, MoexError> {
        self.client
            .candle_borders(&self.engine, &self.market, &self.security)
            .await
    }
}

#[cfg(feature = "blocking")]
impl<'a> IndexAnalyticsPages<'a> {
    /// Получить следующую страницу `index_analytics`.
    pub fn next_page(&mut self) -> Result<Option<Vec<IndexAnalytics>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_index_analytics_page(self.indexid, pagination)
            },
            |item| (item.trade_session_date(), item.secid().clone()),
        )
    }

    /// Собрать все страницы `index_analytics` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<IndexAnalytics>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<IndexAnalytics>, MoexError> {
        self.try_collect()
    }
}

#[cfg(feature = "blocking")]
impl<'a> SecuritiesPages<'a> {
    /// Получить следующую страницу `securities`.
    pub fn next_page(&mut self) -> Result<Option<Vec<Security>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_securities_page(self.engine, self.market, self.board, pagination)
            },
            |item| item.secid().clone(),
        )
    }

    /// Собрать все страницы `securities` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<Security>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<Security>, MoexError> {
        self.try_collect()
    }
}

#[cfg(feature = "blocking")]
impl<'a> GlobalSecuritiesPages<'a> {
    /// Получить следующую страницу глобального `securities`.
    pub fn next_page(&mut self) -> Result<Option<Vec<Security>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| self.client.fetch_global_securities_page(pagination),
            |item| item.secid().clone(),
        )
    }

    /// Собрать все страницы глобального `securities` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<Security>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<Security>, MoexError> {
        self.try_collect()
    }
}

#[cfg(all(feature = "blocking", feature = "news"))]
impl<'a> SiteNewsPages<'a> {
    /// Получить следующую страницу `sitenews`.
    pub fn next_page(&mut self) -> Result<Option<Vec<SiteNews>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| self.client.fetch_sitenews_page(pagination),
            SiteNews::id,
        )
    }

    /// Собрать все страницы `sitenews` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<SiteNews>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<SiteNews>, MoexError> {
        self.try_collect()
    }
}

#[cfg(all(feature = "blocking", feature = "news"))]
impl<'a> EventsPages<'a> {
    /// Получить следующую страницу `events`.
    pub fn next_page(&mut self) -> Result<Option<Vec<Event>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| self.client.fetch_events_page(pagination),
            Event::id,
        )
    }

    /// Собрать все страницы `events` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<Event>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<Event>, MoexError> {
        self.try_collect()
    }
}

#[cfg(feature = "blocking")]
impl<'a> MarketSecuritiesPages<'a> {
    /// Получить следующую страницу market-level `securities`.
    pub fn next_page(&mut self) -> Result<Option<Vec<Security>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_market_securities_page(self.engine, self.market, pagination)
            },
            |item| item.secid().clone(),
        )
    }

    /// Собрать все страницы market-level `securities` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<Security>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<Security>, MoexError> {
        self.try_collect()
    }
}

#[cfg(feature = "blocking")]
impl<'a> MarketTradesPages<'a> {
    /// Получить следующую страницу market-level `trades`.
    pub fn next_page(&mut self) -> Result<Option<Vec<Trade>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_market_trades_page(self.engine, self.market, pagination)
            },
            Trade::tradeno,
        )
    }

    /// Собрать все страницы market-level `trades` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<Trade>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<Trade>, MoexError> {
        self.try_collect()
    }
}

#[cfg(feature = "blocking")]
impl<'a> TradesPages<'a> {
    /// Получить следующую страницу `trades`.
    pub fn next_page(&mut self) -> Result<Option<Vec<Trade>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client.fetch_trades_page(
                    self.engine,
                    self.market,
                    self.board,
                    self.security,
                    pagination,
                )
            },
            Trade::tradeno,
        )
    }

    /// Собрать все страницы `trades` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<Trade>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<Trade>, MoexError> {
        self.try_collect()
    }
}

#[cfg(all(feature = "blocking", feature = "history"))]
impl<'a> HistoryPages<'a> {
    /// Получить следующую страницу `history`.
    pub fn next_page(&mut self) -> Result<Option<Vec<HistoryRecord>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client.fetch_history_page(
                    self.engine,
                    self.market,
                    self.board,
                    self.security,
                    pagination,
                )
            },
            HistoryRecord::tradedate,
        )
    }

    /// Собрать все страницы `history` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<HistoryRecord>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<HistoryRecord>, MoexError> {
        self.try_collect()
    }
}

#[cfg(feature = "blocking")]
impl<'a> SecStatsPages<'a> {
    /// Получить следующую страницу `secstats`.
    pub fn next_page(&mut self) -> Result<Option<Vec<SecStat>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client
                    .fetch_secstats_page(self.engine, self.market, pagination)
            },
            |item| (item.secid().clone(), item.boardid().clone()),
        )
    }

    /// Собрать все страницы `secstats` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<SecStat>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<SecStat>, MoexError> {
        self.try_collect()
    }
}

#[cfg(feature = "blocking")]
impl<'a> CandlesPages<'a> {
    /// Получить следующую страницу `candles`.
    pub fn next_page(&mut self) -> Result<Option<Vec<Candle>>, MoexError> {
        next_page_blocking(
            &mut self.pagination,
            |pagination| {
                self.client.fetch_candles_page(
                    self.engine,
                    self.market,
                    self.board,
                    self.security,
                    self.query,
                    pagination,
                )
            },
            Candle::begin,
        )
    }

    /// Собрать все страницы `candles` в один `Vec`.
    pub fn try_collect(mut self) -> Result<Vec<Candle>, MoexError> {
        collect_pages_blocking(|| self.next_page())
    }

    /// Алиас для [`Self::try_collect`].
    pub fn all(self) -> Result<Vec<Candle>, MoexError> {
        self.try_collect()
    }
}

impl<K> PaginationTracker<K> {
    fn new(
        endpoint: impl Into<String>,
        page_limit: NonZeroU32,
        repeat_page_policy: RepeatPagePolicy,
    ) -> Self {
        Self {
            endpoint: endpoint.into().into_boxed_str(),
            page_limit,
            repeat_page_policy,
            start: 0,
            first_key_on_previous_page: None,
            finished: false,
        }
    }

    fn next_page_request(&self) -> Option<Pagination> {
        if self.finished {
            return None;
        }
        Some(Pagination {
            start: Some(self.start),
            limit: Some(self.page_limit),
        })
    }
}

impl<K> PaginationTracker<K>
where
    K: Eq,
{
    fn advance(
        &mut self,
        page_len: usize,
        first_key_on_page: Option<K>,
    ) -> Result<PaginationAdvance, MoexError> {
        let page_limit = self.page_limit.get();

        if page_len == 0 {
            self.finished = true;
            return Ok(PaginationAdvance::EndOfPages);
        }

        if let (Some(prev), Some(current)) = (&self.first_key_on_previous_page, &first_key_on_page)
            && prev == current
        {
            return match self.repeat_page_policy {
                RepeatPagePolicy::Error => Err(MoexError::PaginationStuck {
                    endpoint: self.endpoint.clone(),
                    start: self.start,
                    limit: page_limit,
                }),
            };
        }

        self.first_key_on_previous_page = first_key_on_page;

        if (page_len as u128) < u128::from(page_limit) {
            self.finished = true;
            return Ok(PaginationAdvance::YieldPage);
        }

        self.start =
            self.start
                .checked_add(page_limit)
                .ok_or_else(|| MoexError::PaginationOverflow {
                    endpoint: self.endpoint.clone(),
                    start: self.start,
                    limit: page_limit,
                })?;

        Ok(PaginationAdvance::YieldPage)
    }
}

#[cfg(feature = "blocking")]
impl<'a> OwnedIndexScope<'a> {
    /// Идентификатор индекса текущего owning-scope.
    pub fn indexid(&self) -> &IndexId {
        &self.indexid
    }

    /// Получить состав индекса (`analytics`) по текущему owning-scope.
    pub fn analytics(&self, page_request: PageRequest) -> Result<Vec<IndexAnalytics>, MoexError> {
        self.client
            .index_analytics_query(&self.indexid, page_request)
    }

    /// Создать ленивый paginator страниц `analytics` для текущего индекса.
    pub fn analytics_pages(&self, page_limit: NonZeroU32) -> IndexAnalyticsPages<'_> {
        self.client.index_analytics_pages(&self.indexid, page_limit)
    }
}

#[cfg(feature = "blocking")]
impl<'a> OwnedEngineScope<'a> {
    /// Имя торгового движка текущего owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Получить доступные рынки (`markets`) для текущего движка.
    pub fn markets(&self) -> Result<Vec<Market>, MoexError> {
        self.client.markets(&self.engine)
    }

    /// Получить обороты (`turnovers`) для текущего движка.
    pub fn turnovers(&self) -> Result<Vec<Turnover>, MoexError> {
        self.client.engine_turnovers(&self.engine)
    }

    /// Зафиксировать рынок внутри текущего `engine`.
    pub fn market<M>(self, market: M) -> Result<OwnedMarketScope<'a>, ParseMarketNameError>
    where
        M: TryInto<MarketName>,
        M::Error: Into<ParseMarketNameError>,
    {
        let market = market.try_into().map_err(Into::into)?;
        Ok(OwnedMarketScope {
            client: self.client,
            engine: self.engine,
            market,
        })
    }

    /// Shortcut для часто используемого рынка `shares`.
    pub fn shares(self) -> Result<OwnedMarketScope<'a>, ParseMarketNameError> {
        self.market("shares")
    }
}

#[cfg(feature = "blocking")]
impl<'a> OwnedMarketScope<'a> {
    /// Имя торгового движка текущего owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Имя рынка текущего owning-scope.
    pub fn market(&self) -> &MarketName {
        &self.market
    }

    /// Получить режимы торгов (`boards`) для текущего рынка.
    pub fn boards(&self) -> Result<Vec<Board>, MoexError> {
        self.client.boards(&self.engine, &self.market)
    }

    /// Получить инструменты (`securities`) на уровне текущего рынка.
    pub fn securities(&self, page_request: PageRequest) -> Result<Vec<Security>, MoexError> {
        self.client
            .market_securities_query(&self.engine, &self.market, page_request)
    }

    /// Создать ленивый paginator страниц market-level `securities`.
    pub fn securities_pages(&self, page_limit: NonZeroU32) -> MarketSecuritiesPages<'_> {
        self.client
            .market_securities_pages(&self.engine, &self.market, page_limit)
    }

    /// Получить market-level стакан (`orderbook`) для текущего рынка.
    pub fn orderbook(&self) -> Result<Vec<OrderbookLevel>, MoexError> {
        self.client.market_orderbook(&self.engine, &self.market)
    }

    /// Получить market-level сделки (`trades`) для текущего рынка.
    pub fn trades(&self, page_request: PageRequest) -> Result<Vec<Trade>, MoexError> {
        self.client
            .market_trades_query(&self.engine, &self.market, page_request)
    }

    /// Создать ленивый paginator страниц market-level `trades`.
    pub fn trades_pages(&self, page_limit: NonZeroU32) -> MarketTradesPages<'_> {
        self.client
            .market_trades_pages(&self.engine, &self.market, page_limit)
    }

    /// Получить `secstats` для текущего рынка.
    pub fn secstats(&self, page_request: PageRequest) -> Result<Vec<SecStat>, MoexError> {
        self.client
            .secstats_query(&self.engine, &self.market, page_request)
    }

    /// Создать ленивый paginator страниц `secstats`.
    pub fn secstats_pages(&self, page_limit: NonZeroU32) -> SecStatsPages<'_> {
        self.client
            .secstats_pages(&self.engine, &self.market, page_limit)
    }

    /// Получить доступные границы свечей (`candleborders`) по инструменту.
    pub fn candle_borders(&self, security: &SecId) -> Result<Vec<CandleBorder>, MoexError> {
        self.client
            .candle_borders(&self.engine, &self.market, security)
    }

    /// Зафиксировать инструмент в рамках текущего `engine/market`.
    pub fn security<S>(self, security: S) -> Result<OwnedMarketSecurityScope<'a>, ParseSecIdError>
    where
        S: TryInto<SecId>,
        S::Error: Into<ParseSecIdError>,
    {
        let security = security.try_into().map_err(Into::into)?;
        Ok(OwnedMarketSecurityScope {
            client: self.client,
            engine: self.engine,
            market: self.market,
            security,
        })
    }

    /// Зафиксировать `board` внутри текущего `engine/market`.
    pub fn board<B>(self, board: B) -> Result<OwnedBoardScope<'a>, ParseBoardIdError>
    where
        B: TryInto<BoardId>,
        B::Error: Into<ParseBoardIdError>,
    {
        let board = board.try_into().map_err(Into::into)?;
        Ok(OwnedBoardScope {
            client: self.client,
            engine: self.engine,
            market: self.market,
            board,
        })
    }
}

#[cfg(feature = "blocking")]
impl<'a> OwnedBoardScope<'a> {
    /// Имя торгового движка текущего owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Имя рынка текущего owning-scope.
    pub fn market(&self) -> &MarketName {
        &self.market
    }

    /// Идентификатор режима торгов текущего owning-scope.
    pub fn board(&self) -> &BoardId {
        &self.board
    }

    /// Получить инструменты (`securities`) по текущему owning-scope.
    pub fn securities(&self, page_request: PageRequest) -> Result<Vec<Security>, MoexError> {
        self.client
            .securities_query(&self.engine, &self.market, &self.board, page_request)
    }

    /// Создать ленивый paginator страниц `securities` по текущему owning-scope.
    pub fn securities_pages(&self, page_limit: NonZeroU32) -> SecuritiesPages<'_> {
        self.client
            .securities_pages(&self.engine, &self.market, &self.board, page_limit)
    }

    /// Получить снимки инструментов (`LOTSIZE` и `LAST`) для текущего owning-scope.
    pub fn snapshots(&self) -> Result<Vec<SecuritySnapshot>, MoexError> {
        self.client
            .board_snapshots(&self.engine, &self.market, &self.board)
    }

    /// Зафиксировать инструмент в рамках текущего `engine/market/board`.
    pub fn security<S>(self, security: S) -> Result<OwnedSecurityScope<'a>, ParseSecIdError>
    where
        S: TryInto<SecId>,
        S::Error: Into<ParseSecIdError>,
    {
        let security = security.try_into().map_err(Into::into)?;
        Ok(OwnedSecurityScope {
            client: self.client,
            engine: self.engine,
            market: self.market,
            board: self.board,
            security,
        })
    }
}

#[cfg(feature = "blocking")]
impl<'a> OwnedSecurityResourceScope<'a> {
    /// Идентификатор инструмента текущего owning-scope.
    pub fn secid(&self) -> &SecId {
        &self.security
    }

    /// Получить карточку текущего инструмента.
    pub fn info(&self) -> Result<Option<Security>, MoexError> {
        self.client.security_info(&self.security)
    }

    /// Получить режимы торгов (`boards`) для текущего инструмента.
    pub fn boards(&self) -> Result<Vec<SecurityBoard>, MoexError> {
        self.client.security_boards(&self.security)
    }
}

#[cfg(feature = "blocking")]
impl<'a> OwnedSecurityScope<'a> {
    /// Идентификатор инструмента текущего owning-scope.
    pub fn security(&self) -> &SecId {
        &self.security
    }

    /// Получить стакан (`orderbook`) по текущему инструменту.
    pub fn orderbook(&self) -> Result<Vec<OrderbookLevel>, MoexError> {
        self.client
            .orderbook(&self.engine, &self.market, &self.board, &self.security)
    }

    #[cfg(feature = "history")]
    /// Получить диапазон доступных исторических дат по текущему инструменту.
    pub fn history_dates(&self) -> Result<Option<HistoryDates>, MoexError> {
        self.client
            .history_dates(&self.engine, &self.market, &self.board, &self.security)
    }

    #[cfg(feature = "history")]
    /// Получить исторические данные (`history`) по текущему инструменту.
    pub fn history(&self, page_request: PageRequest) -> Result<Vec<HistoryRecord>, MoexError> {
        self.client.history_query(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            page_request,
        )
    }

    #[cfg(feature = "history")]
    /// Создать ленивый paginator страниц `history` по текущему инструменту.
    pub fn history_pages(&self, page_limit: NonZeroU32) -> HistoryPages<'_> {
        self.client.history_pages(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            page_limit,
        )
    }

    /// Получить доступные границы свечей (`candleborders`) по текущему инструменту.
    pub fn candle_borders(&self) -> Result<Vec<CandleBorder>, MoexError> {
        self.client
            .candle_borders(&self.engine, &self.market, &self.security)
    }

    /// Получить сделки (`trades`) по текущему инструменту.
    pub fn trades(&self, page_request: PageRequest) -> Result<Vec<Trade>, MoexError> {
        self.client.trades_query(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            page_request,
        )
    }

    /// Создать ленивый paginator страниц `trades` по текущему инструменту.
    pub fn trades_pages(&self, page_limit: NonZeroU32) -> TradesPages<'_> {
        self.client.trades_pages(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            page_limit,
        )
    }

    /// Получить свечи (`candles`) по текущему инструменту.
    pub fn candles(
        &self,
        query: CandleQuery,
        page_request: PageRequest,
    ) -> Result<Vec<Candle>, MoexError> {
        self.client.candles_query(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            query,
            page_request,
        )
    }

    /// Создать ленивый paginator `candles` по текущему инструменту.
    pub fn candles_pages(&self, query: CandleQuery, page_limit: NonZeroU32) -> CandlesPages<'_> {
        self.client.candles_pages(
            &self.engine,
            &self.market,
            &self.board,
            &self.security,
            query,
            page_limit,
        )
    }
}

#[cfg(feature = "blocking")]
impl<'a> OwnedMarketSecurityScope<'a> {
    /// Имя торгового движка текущего owning-scope.
    pub fn engine(&self) -> &EngineName {
        &self.engine
    }

    /// Имя рынка текущего owning-scope.
    pub fn market(&self) -> &MarketName {
        &self.market
    }

    /// Идентификатор инструмента текущего owning-scope.
    pub fn security(&self) -> &SecId {
        &self.security
    }

    /// Получить карточку текущего инструмента на уровне рынка.
    pub fn info(&self) -> Result<Option<Security>, MoexError> {
        self.client
            .market_security_info(&self.engine, &self.market, &self.security)
    }

    /// Получить доступные границы свечей (`candleborders`) по текущему инструменту.
    pub fn candle_borders(&self) -> Result<Vec<CandleBorder>, MoexError> {
        self.client
            .candle_borders(&self.engine, &self.market, &self.security)
    }
}

fn apply_iss_request_options(query: &mut Vec<(Box<str>, Box<str>)>, options: IssRequestOptions) {
    if let Some(metadata) = options.metadata_value() {
        query.push((ISS_META_PARAM.into(), metadata.as_query_value().into()));
    }
    if let Some(data) = options.data_value() {
        query.push((ISS_DATA_PARAM.into(), data.as_query_value().into()));
    }
    if let Some(version) = options.version_value() {
        query.push((ISS_VERSION_PARAM.into(), version.as_query_value().into()));
    }
    if let Some(json) = options.json_value() {
        query.push((ISS_JSON_PARAM.into(), json.into()));
    }
}

/// Нормализовать raw endpoint-path к виду `relative/path.json`.
///
/// Запрещает query-string в пути и позволяет передавать как `iss/...`,
/// так и путь без префикса.
pub(super) fn normalize_raw_endpoint_path(path: Option<&str>) -> Result<Box<str>, MoexError> {
    let raw = path.ok_or(MoexError::MissingRawPath)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(MoexError::InvalidRawPath {
            path: raw.to_owned().into_boxed_str(),
            reason: "path must not be empty".into(),
        });
    }
    if trimmed.contains('?') {
        return Err(MoexError::InvalidRawPath {
            path: raw.to_owned().into_boxed_str(),
            reason: "query string is not allowed in path; use .param(...)".into(),
        });
    }

    // Поддерживаем пути вида `/iss/...` и `iss/...`, чтобы API builder-а был гибким.
    let without_slash = trimmed.trim_start_matches('/');
    let endpoint = without_slash
        .strip_prefix("iss/")
        .unwrap_or(without_slash)
        .trim();

    if endpoint.is_empty() {
        return Err(MoexError::InvalidRawPath {
            path: raw.to_owned().into_boxed_str(),
            reason: "endpoint path is empty after normalization".into(),
        });
    }

    if endpoint.ends_with(".json") {
        return Ok(endpoint.to_owned().into_boxed_str());
    }

    let mut normalized = endpoint.to_owned();
    normalized.push_str(".json");
    Ok(normalized.into_boxed_str())
}

/// Преобразовать список `securities/{secid}` в опциональную единственную запись.
pub(super) fn optional_single_security(
    endpoint: &str,
    mut securities: Vec<Security>,
) -> Result<Option<Security>, MoexError> {
    if securities.len() > 1 {
        return Err(MoexError::UnexpectedSecurityRows {
            endpoint: endpoint.to_owned().into_boxed_str(),
            row_count: securities.len(),
        });
    }
    Ok(securities.pop())
}

#[cfg(feature = "history")]
/// Преобразовать список `history/.../dates` в опциональную единственную запись.
pub(super) fn optional_single_history_dates(
    endpoint: &str,
    mut dates: Vec<HistoryDates>,
) -> Result<Option<HistoryDates>, MoexError> {
    if dates.len() > 1 {
        return Err(MoexError::UnexpectedHistoryDatesRows {
            endpoint: endpoint.to_owned().into_boxed_str(),
            row_count: dates.len(),
        });
    }
    Ok(dates.pop())
}

/// Добавить параметры запроса свечей (`from`, `till`, `interval`) в URL.
pub(super) fn append_candle_query_to_url(endpoint_url: &mut Url, candle_query: CandleQuery) {
    let mut query_pairs = endpoint_url.query_pairs_mut();
    if let Some(from) = candle_query.from() {
        let from = from.format("%Y-%m-%d %H:%M:%S").to_string();
        query_pairs.append_pair(FROM_PARAM, &from);
    }
    if let Some(till) = candle_query.till() {
        let till = till.format("%Y-%m-%d %H:%M:%S").to_string();
        query_pairs.append_pair(TILL_PARAM, &till);
    }
    if let Some(interval) = candle_query.interval() {
        query_pairs.append_pair(INTERVAL_PARAM, interval.as_str());
    }
}

/// Добавить параметры пагинации ISS (`start`, `limit`) в URL.
pub(super) fn append_pagination_to_url(endpoint_url: &mut Url, pagination: Pagination) {
    if pagination.start.is_none() && pagination.limit.is_none() {
        return;
    }

    let mut query = endpoint_url.query_pairs_mut();
    if let Some(start) = pagination.start {
        let start = start.to_string();
        query.append_pair(START_PARAM, &start);
    }
    if let Some(limit) = pagination.limit {
        let limit = limit.get().to_string();
        query.append_pair(LIMIT_PARAM, &limit);
    }
}

/// Быстрая эвристика, похож ли ответ на JSON.
pub(super) fn looks_like_json_payload(content_type: Option<&str>, payload: &str) -> bool {
    if content_type.is_some_and(contains_json_token_ascii_case_insensitive) {
        return true;
    }

    let trimmed = payload.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn contains_json_token_ascii_case_insensitive(content_type: &str) -> bool {
    content_type
        .as_bytes()
        .windows(4)
        .any(|window| window.eq_ignore_ascii_case(b"json"))
}

/// Взять безопасный префикс payload для диагностических сообщений.
pub(super) fn truncate_prefix(payload: &str, max_chars: usize) -> Box<str> {
    payload
        .chars()
        .take(max_chars)
        .collect::<String>()
        .into_boxed_str()
}

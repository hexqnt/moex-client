//! Настройка ISS-запросов, повторы и ограничение частоты.

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use reqwest::{StatusCode, header::HeaderMap};

use super::MoexError;
use super::constants::{ISS_DATA_PARAM, ISS_JSON_PARAM, ISS_META_PARAM, ISS_VERSION_PARAM};

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

/// Политика повторных попыток для операций с [`MoexError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct RetryPolicy {
    max_attempts: NonZeroU32,
    delay: Duration,
}

impl RetryPolicy {
    /// Создать политику повторов с заданным числом попыток и паузой по умолчанию.
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
        Self::new(const { NonZeroU32::new(3).unwrap() })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Ограничение частоты запросов.
///
/// Хранит минимальный интервал между последовательными запросами.
#[must_use]
pub struct RateLimit {
    min_interval: Duration,
}

impl RateLimit {
    /// Создать ограничение из минимального интервала между запросами.
    pub fn every(min_interval: Duration) -> Self {
        Self { min_interval }
    }

    /// Создать ограничение из числа запросов в секунду.
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
/// Состояние rate limit для последовательности запросов.
pub struct RateLimiter {
    limit: RateLimit,
    next_allowed_at: Option<Instant>,
}

impl RateLimiter {
    /// Создать новый ограничитель с заданным лимитом.
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

    pub(super) fn reserve_delay_at(&mut self, now: Instant) -> Duration {
        let scheduled_at = match self.next_allowed_at {
            Some(next_allowed_at) if next_allowed_at > now => next_allowed_at,
            _ => now,
        };
        let delay = scheduled_at.saturating_duration_since(now);
        self.next_allowed_at = Some(scheduled_at + self.limit.min_interval);
        delay
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Системные опции ISS-запроса (`iss.*`) для raw endpoint-ов.
#[must_use]
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

    pub(super) fn append_to(self, query: &mut Vec<(Box<str>, Box<str>)>) {
        if let Some(metadata) = self.metadata {
            query.push((ISS_META_PARAM.into(), metadata.as_query_value().into()));
        }
        if let Some(data) = self.data {
            query.push((ISS_DATA_PARAM.into(), data.as_query_value().into()));
        }
        if let Some(version) = self.version {
            query.push((ISS_VERSION_PARAM.into(), version.as_query_value().into()));
        }
        if let Some(json) = self.json {
            query.push((ISS_JSON_PARAM.into(), json));
        }
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
pub struct RawIssResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

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

macro_rules! retry_loop {
    ($policy:ident, $action:expr, $sleep:expr) => {{
        let mut attempts_left = $policy.max_attempts().get();
        loop {
            match $action {
                Ok(value) => return Ok(value),
                Err(error) if attempts_left > 1 && error.is_retryable() => {
                    attempts_left -= 1;
                    $sleep;
                }
                Err(error) => return Err(error),
            }
        }
    }};
}

/// Выполнить блокирующую операцию с повторами retryable-ошибок.
///
/// Повтор выполняется только для [`MoexError::is_retryable`].
pub fn with_retry<T, F>(policy: RetryPolicy, mut action: F) -> Result<T, MoexError>
where
    F: FnMut() -> Result<T, MoexError>,
{
    retry_loop!(policy, action(), std::thread::sleep(policy.delay()))
}

/// Выполнить блокирующую операцию с соблюдением [`RateLimiter`].
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

/// Выполнить асинхронную операцию с повторами retryable-ошибок.
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
    retry_loop!(policy, action().await, sleep(policy.delay()).await)
}

/// Выполнить асинхронную операцию с соблюдением [`RateLimiter`].
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

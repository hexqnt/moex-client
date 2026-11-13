use super::*;

macro_rules! client_constructors {
    ($http_client:ty) => {
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
        pub fn with_client(client: $http_client) -> Result<Self, MoexError> {
            Self::builder().client(client).build()
        }

        /// Создать клиент на базе переданного `reqwest`-клиента и включить `iss.meta`.
        pub fn with_client_with_metadata(client: $http_client) -> Result<Self, MoexError> {
            Self::builder().metadata(true).client(client).build()
        }

        /// Создать клиент с явным базовым URL и готовым HTTP-клиентом (`iss.meta=off`).
        pub fn with_base_url(client: $http_client, base_url: Url) -> Self {
            Self::with_base_url_and_rate_limit(client, base_url, false, None)
        }

        /// Создать клиент с явным базовым URL, готовым HTTP-клиентом и `iss.meta=on`.
        pub fn with_base_url_with_metadata(client: $http_client, base_url: Url) -> Self {
            Self::with_base_url_and_rate_limit(client, base_url, true, None)
        }
    };
}

macro_rules! client_scopes {
    ($engine_scope:ident, $index_scope:ident, $security_scope:ident) => {
        /// Зафиксировать контекст `engine` из значения, реализующего `TryInto<EngineName>`.
        pub fn engine<E>(&self, engine: E) -> Result<$engine_scope<'_>, ParseEngineNameError>
        where
            E: TryInto<EngineName>,
            E::Error: Into<ParseEngineNameError>,
        {
            let engine = engine.try_into().map_err(Into::into)?;
            Ok($engine_scope {
                client: self,
                engine,
            })
        }

        /// Сокращение для часто используемого движка `stock`.
        pub fn stock(&self) -> Result<$engine_scope<'_>, ParseEngineNameError> {
            self.engine("stock")
        }

        /// Зафиксировать контекст `indexid` из значения, реализующего `TryInto<IndexId>`.
        pub fn index<I>(&self, indexid: I) -> Result<$index_scope<'_>, ParseIndexError>
        where
            I: TryInto<IndexId>,
            I::Error: Into<ParseIndexError>,
        {
            let indexid = indexid.try_into().map_err(Into::into)?;
            Ok($index_scope {
                client: self,
                indexid,
            })
        }

        /// Зафиксировать контекст `secid` из значения, реализующего `TryInto<SecId>`.
        pub fn security<S>(&self, security: S) -> Result<$security_scope<'_>, ParseSecIdError>
        where
            S: TryInto<SecId>,
            S::Error: Into<ParseSecIdError>,
        {
            let security = security.try_into().map_err(Into::into)?;
            Ok($security_scope {
                client: self,
                security,
            })
        }
    };
}

macro_rules! client_requests {
    ($request:ident, $response:ty, $mode:ident, [$($asyncness:tt)?]) => {
        /// Создать builder raw-запроса для произвольного ISS endpoint.
        pub fn raw(&self) -> $request<'_> {
            $request {
                client: self,
                path: None,
                query: Vec::new(),
            }
        }

        /// Создать builder raw-запроса для типизированного ISS endpoint-а.
        ///
        /// Запрос автоматически получает `path` и значение `iss.only` по умолчанию.
        pub fn raw_endpoint(&self, endpoint: IssEndpoint<'_>) -> $request<'_> {
            let path = endpoint.path();
            let request = self.raw().path(path);
            match endpoint.default_table() {
                Some(table) => request.only(table),
                None => request,
            }
        }

        /// Получить снимки инструментов (`LOTSIZE` и `LAST`) по данным `SecurityBoard`.
        pub $($asyncness)? fn board_security_snapshots(
            &self,
            board: &SecurityBoard,
        ) -> Result<Vec<SecuritySnapshot>, MoexError> {
            await_result!(
                $mode,
                self.board_snapshots(board.engine(), board.market(), board.boardid())
            )
        }

        pub(super) $($asyncness)? fn fetch_response(
            &self,
            endpoint: &str,
            endpoint_url: Url,
        ) -> Result<$response, MoexError> {
            await_result!($mode, self.wait_for_rate_limit());
            await_result!($mode, self.client.get(endpoint_url).send())
                .map_err(|source| MoexError::Request {
                    endpoint: endpoint.to_owned().into_boxed_str(),
                    source,
                })
        }

        pub(super) $($asyncness)? fn fetch_payload(
            &self,
            endpoint: &str,
            endpoint_url: Url,
        ) -> Result<String, MoexError> {
            let response = await_result!($mode, self.fetch_response(endpoint, endpoint_url))?;
            let status = response.status();
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .cloned();
            let payload = await_result!($mode, response.text()).map_err(|source| MoexError::ReadBody {
                endpoint: endpoint.to_owned().into_boxed_str(),
                source,
            })?;
            validate_payload(endpoint, status, content_type, payload)
        }
    };
}

macro_rules! history_defaults {
    ($mode:ident, [$($asyncness:tt)?], $pages:ident) => {
        #[cfg(feature = "history")]
        /// Получить исторические данные (`history`) с единым режимом выборки страниц.
        pub $($asyncness)? fn history_query(
            &self,
            engine: &EngineName,
            market: &MarketName,
            board: &BoardId,
            security: &SecId,
            page_request: PageRequest,
        ) -> Result<Vec<HistoryRecord>, MoexError> {
            await_result!(
                $mode,
                self.history_query_filtered(
                    engine,
                    market,
                    board,
                    security,
                    HistoryQuery::default(),
                    page_request,
                )
            )
        }

        #[cfg(feature = "history")]
        /// Создать ленивый пагинатор страниц `history`.
        pub fn history_pages<'a>(
            &'a self,
            engine: &'a EngineName,
            market: &'a MarketName,
            board: &'a BoardId,
            security: &'a SecId,
            page_limit: NonZeroU32,
        ) -> $pages<'a> {
            self.history_pages_filtered(
                engine,
                market,
                board,
                security,
                HistoryQuery::default(),
                page_limit,
            )
        }
    };
}

/// Блокирующий клиент ISS API Московской биржи.
#[cfg(feature = "blocking")]
pub struct BlockingMoexClient {
    pub(super) base_url: Url,
    pub(super) metadata: bool,
    pub(super) client: Client,
    pub(super) rate_limiter: Option<Mutex<RateLimiter>>,
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

    client_constructors!(Client);

    /// Текущее ограничение частоты запросов, если оно включено.
    pub fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limiter
            .as_ref()
            .map(|limiter| lock_rate_limiter(limiter).limit())
    }

    pub(super) fn with_base_url_and_rate_limit(
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

    client_requests!(
        RawIssRequestBuilder,
        reqwest::blocking::Response,
        blocking,
        []
    );

    client_scopes!(
        OwnedEngineScope,
        OwnedIndexScope,
        OwnedSecurityResourceScope
    );

    history_defaults!(blocking, [], HistoryPages);

    pub(super) fn wait_for_rate_limit(&self) {
        let Some(limiter) = &self.rate_limiter else {
            return;
        };
        let delay = reserve_rate_limit_delay(limiter);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

/// Асинхронный клиент ISS API Московской биржи.
#[cfg(feature = "async")]
pub struct AsyncMoexClient {
    pub(super) base_url: Url,
    pub(super) metadata: bool,
    pub(super) client: reqwest::Client,
    pub(super) rate_limit: Option<AsyncRateLimitState>,
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

    client_constructors!(reqwest::Client);

    /// Текущее ограничение частоты запросов, если оно включено.
    pub fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
            .as_ref()
            .map(|rate_limit| lock_rate_limiter(&rate_limit.limiter).limit())
    }

    pub(super) fn with_base_url_and_rate_limit(
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

    client_requests!(AsyncRawIssRequestBuilder, reqwest::Response, asynchronous, [async]);

    client_scopes!(
        AsyncOwnedEngineScope,
        AsyncOwnedIndexScope,
        AsyncOwnedSecurityResourceScope
    );

    history_defaults!(asynchronous, [async], AsyncHistoryPages);

    pub(super) async fn wait_for_rate_limit(&self) {
        let Some(rate_limit) = &self.rate_limit else {
            return;
        };
        let delay = reserve_rate_limit_delay(&rate_limit.limiter);
        if !delay.is_zero() {
            (rate_limit.sleep)(delay).await;
        }
    }
}

#[cfg(feature = "async")]
pub(super) struct AsyncRateLimitState {
    limiter: Mutex<RateLimiter>,
    sleep: AsyncRateLimitSleep,
}

pub(super) fn resolve_base_url_or_default(base_url: Option<Url>) -> Result<Url, MoexError> {
    match base_url {
        Some(base_url) => Ok(base_url),
        None => Url::parse(BASE_URL).map_err(|source| MoexError::InvalidBaseUrl {
            base_url: BASE_URL,
            reason: source.to_string(),
        }),
    }
}

pub(super) fn resolve_http_client<T>(
    client: Option<T>,
    build: impl FnOnce() -> Result<T, reqwest::Error>,
) -> Result<T, MoexError> {
    match client {
        Some(client) => Ok(client),
        None => build().map_err(|source| MoexError::BuildHttpClient { source }),
    }
}

#[cfg(feature = "async")]
pub(super) fn resolve_async_rate_limit_state(
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

pub(super) fn lock_rate_limiter(
    limiter: &Mutex<RateLimiter>,
) -> std::sync::MutexGuard<'_, RateLimiter> {
    match limiter.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub(super) fn reserve_rate_limit_delay(limiter: &Mutex<RateLimiter>) -> Duration {
    let mut limiter = lock_rate_limiter(limiter);
    limiter.reserve_delay()
}

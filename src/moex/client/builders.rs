use super::*;

macro_rules! builder_settings {
    ($http_client:ty) => {
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

        /// Передать готовый `reqwest`-клиент.
        pub fn client(mut self, client: $http_client) -> Self {
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
    };
}

/// Builder для конфигурации [`BlockingMoexClient`].
#[cfg(feature = "blocking")]
#[must_use]
pub struct BlockingMoexClientBuilder {
    pub(super) base_url: Option<Url>,
    pub(super) metadata: bool,
    pub(super) client: Option<Client>,
    pub(super) http_client: ClientBuilder,
    pub(super) rate_limit: Option<RateLimit>,
}

#[cfg(feature = "blocking")]
impl BlockingMoexClientBuilder {
    builder_settings!(Client);

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

/// Builder для конфигурации [`AsyncMoexClient`].
#[cfg(feature = "async")]
#[must_use]
pub struct AsyncMoexClientBuilder {
    pub(super) base_url: Option<Url>,
    pub(super) metadata: bool,
    pub(super) client: Option<reqwest::Client>,
    pub(super) http_client: reqwest::ClientBuilder,
    pub(super) rate_limit: Option<RateLimit>,
    pub(super) rate_limit_sleep: Option<AsyncRateLimitSleep>,
}

#[cfg(feature = "async")]
impl AsyncMoexClientBuilder {
    builder_settings!(reqwest::Client);

    /// Задать async-функцию ожидания для использования с [`Self::rate_limit`].
    ///
    /// Обязательна при включённом ограничении частоты запросов.
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

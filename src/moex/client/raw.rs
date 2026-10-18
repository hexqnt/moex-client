use super::*;

macro_rules! raw_builder {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?]) => {
        /// Универсальный builder для произвольных ISS endpoint-ов.
        #[must_use]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) path: Option<Box<str>>,
            pub(super) query: Vec<(Box<str>, Box<str>)>,
        }

        impl<'a> $name<'a> {
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
            pub $($asyncness)? fn send_response(self) -> Result<RawIssResponse, MoexError> {
                let (endpoint, endpoint_url) = self.build_request()?;
                let response = await_result!(
                    $mode,
                    self.client.fetch_response(&endpoint, endpoint_url)
                )?;
                let status = response.status();
                let headers = response.headers().clone();
                let body = await_result!($mode, response.text())
                    .map_err(|source| MoexError::ReadBody { endpoint, source })?;
                Ok(RawIssResponse::new(status, headers, body))
            }

            /// Выполнить raw-запрос и вернуть тело ответа как строку.
            pub $($asyncness)? fn send_payload(self) -> Result<String, MoexError> {
                let (_, payload) = await_result!($mode, self.execute())?;
                Ok(payload)
            }

            /// Выполнить raw-запрос и декодировать JSON в пользовательский тип.
            pub $($asyncness)? fn send_json<T>(self) -> Result<T, MoexError>
            where
                T: serde::de::DeserializeOwned,
            {
                let (endpoint, payload) = await_result!($mode, self.execute())?;
                serde_json::from_str(&payload).map_err(|source| MoexError::Decode { endpoint, source })
            }

            /// Выполнить raw-запрос и декодировать строки выбранной ISS-таблицы в пользовательский тип.
            pub $($asyncness)? fn send_table<T>(self, table: impl Into<String>) -> Result<Vec<T>, MoexError>
            where
                T: serde::de::DeserializeOwned,
            {
                let table = table.into();
                let (endpoint, payload) = await_result!($mode, self.execute())?;
                decode_raw_table_rows_json_with_endpoint(&payload, endpoint.as_ref(), table.as_str())
            }

            $($asyncness)? fn execute(self) -> Result<(Box<str>, String), MoexError> {
                let (endpoint, endpoint_url) = self.build_request()?;
                let payload = await_result!(
                    $mode,
                    self.client.fetch_payload(&endpoint, endpoint_url)
                )?;
                Ok((endpoint, payload))
            }

            pub(super) fn build_request(&self) -> Result<(Box<str>, Url), MoexError> {
                build_raw_request(
                    &self.client.base_url,
                    self.client.metadata,
                    self.path.as_deref(),
                    &self.query,
                )
            }
        }
    };
}

#[cfg(feature = "blocking")]
raw_builder!(RawIssRequestBuilder, BlockingMoexClient, blocking, []);
#[cfg(feature = "async")]
raw_builder!(AsyncRawIssRequestBuilder, AsyncMoexClient, asynchronous, [async]);

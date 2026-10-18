use super::*;

#[derive(Clone, Copy)]
pub(super) enum TableSelection {
    Indices,
    Analytics,
    Turnovers,
    Engines,
    Markets,
    Boards,
    SecurityBoards,
    Securities,
    Snapshots,
    Orderbook,
    Candles,
    Trades,
    SecStats,
    MetadataOnly,
    #[cfg(feature = "news")]
    SiteNews,
    #[cfg(feature = "news")]
    Events,
    #[cfg(feature = "history")]
    HistoryDates,
    #[cfg(feature = "history")]
    History,
}

impl TableSelection {
    fn parameters(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Indices => &[
                (ISS_ONLY_PARAM, "indices"),
                (INDICES_COLUMNS_PARAM, INDICES_COLUMNS),
            ],
            Self::Analytics => &[
                (ISS_ONLY_PARAM, "analytics"),
                (ANALYTICS_COLUMNS_PARAM, ANALYTICS_COLUMNS),
            ],
            Self::Turnovers => &[
                (ISS_ONLY_PARAM, "turnovers"),
                (TURNOVERS_COLUMNS_PARAM, TURNOVERS_COLUMNS),
            ],
            Self::Engines => &[
                (ISS_ONLY_PARAM, "engines"),
                (ENGINES_COLUMNS_PARAM, ENGINES_COLUMNS),
            ],
            Self::Markets => &[
                (ISS_ONLY_PARAM, "markets"),
                (MARKETS_COLUMNS_PARAM, MARKETS_COLUMNS),
            ],
            Self::Boards => &[
                (ISS_ONLY_PARAM, "boards"),
                (BOARDS_COLUMNS_PARAM, BOARDS_COLUMNS),
            ],
            Self::SecurityBoards => &[
                (ISS_ONLY_PARAM, "boards"),
                (BOARDS_COLUMNS_PARAM, SECURITY_BOARDS_COLUMNS),
            ],
            Self::Securities => &[
                (ISS_ONLY_PARAM, "securities"),
                (SECURITIES_COLUMNS_PARAM, SECURITIES_COLUMNS),
            ],
            Self::Snapshots => &[
                (ISS_ONLY_PARAM, "securities,marketdata"),
                (SECURITIES_COLUMNS_PARAM, SECURITIES_SNAPSHOT_COLUMNS),
                (MARKETDATA_COLUMNS_PARAM, MARKETDATA_LAST_COLUMNS),
            ],
            Self::Orderbook => &[
                (ISS_ONLY_PARAM, "orderbook"),
                (ORDERBOOK_COLUMNS_PARAM, ORDERBOOK_COLUMNS),
            ],
            Self::Candles => &[
                (ISS_ONLY_PARAM, "candles"),
                (CANDLES_COLUMNS_PARAM, CANDLES_COLUMNS),
            ],
            Self::Trades => &[
                (ISS_ONLY_PARAM, "trades"),
                (TRADES_COLUMNS_PARAM, TRADES_COLUMNS),
            ],
            Self::SecStats => &[
                (ISS_ONLY_PARAM, "secstats"),
                (SECSTATS_COLUMNS_PARAM, SECSTATS_COLUMNS),
            ],
            Self::MetadataOnly => &[],
            #[cfg(feature = "news")]
            Self::SiteNews => &[
                (ISS_ONLY_PARAM, "sitenews"),
                (SITENEWS_COLUMNS_PARAM, SITENEWS_COLUMNS),
            ],
            #[cfg(feature = "news")]
            Self::Events => &[
                (ISS_ONLY_PARAM, "events"),
                (EVENTS_COLUMNS_PARAM, EVENTS_COLUMNS),
            ],
            #[cfg(feature = "history")]
            Self::HistoryDates => &[(ISS_ONLY_PARAM, "dates")],
            #[cfg(feature = "history")]
            Self::History => &[
                (ISS_ONLY_PARAM, "history"),
                (HISTORY_COLUMNS_PARAM, HISTORY_COLUMNS),
            ],
        }
    }
}

pub(super) fn apply_iss_request_options(
    query: &mut Vec<(Box<str>, Box<str>)>,
    options: IssRequestOptions,
) {
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

pub(in crate::moex) fn normalize_raw_endpoint_path(
    path: Option<&str>,
) -> Result<Box<str>, MoexError> {
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

    // Поддерживаем пути вида `/iss/...` и `iss/...`, чтобы API builder'а оставался гибким.
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

pub(in crate::moex) fn optional_single_security(
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
pub(in crate::moex) fn optional_single_history_dates(
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

pub(in crate::moex) fn append_candle_query_to_url(
    endpoint_url: &mut Url,
    candle_query: CandleQuery,
) {
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

#[cfg(feature = "history")]
pub(in crate::moex) fn append_history_query_to_url(
    endpoint_url: &mut Url,
    history_query: HistoryQuery,
) {
    let mut query_pairs = endpoint_url.query_pairs_mut();
    if let Some(from) = history_query.from() {
        let from = from.format("%Y-%m-%d").to_string();
        query_pairs.append_pair(FROM_PARAM, &from);
    }
    if let Some(till) = history_query.till() {
        let till = till.format("%Y-%m-%d").to_string();
        query_pairs.append_pair(TILL_PARAM, &till);
    }
}

pub(in crate::moex) fn append_pagination_to_url(endpoint_url: &mut Url, pagination: Pagination) {
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

pub(in crate::moex) fn looks_like_json_payload(content_type: Option<&str>, payload: &str) -> bool {
    if content_type.is_some_and(contains_json_token_ascii_case_insensitive) {
        return true;
    }

    let trimmed = payload.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

pub(super) fn contains_json_token_ascii_case_insensitive(content_type: &str) -> bool {
    content_type
        .as_bytes()
        .windows(4)
        .any(|window| window.eq_ignore_ascii_case(b"json"))
}

pub(in crate::moex) fn truncate_prefix(payload: &str, max_chars: usize) -> Box<str> {
    payload
        .chars()
        .take(max_chars)
        .collect::<String>()
        .into_boxed_str()
}

pub(super) fn endpoint_url(base_url: &Url, endpoint: &str) -> Result<Url, MoexError> {
    base_url
        .join(endpoint)
        .map_err(|source| MoexError::EndpointUrl {
            endpoint: endpoint.to_owned().into_boxed_str(),
            reason: source.to_string(),
        })
}

pub(super) fn table_url(
    base_url: &Url,
    metadata: bool,
    endpoint: &str,
    table: TableSelection,
) -> Result<Url, MoexError> {
    let mut url = endpoint_url(base_url, endpoint)?;
    url.query_pairs_mut()
        .append_pair(ISS_META_PARAM, metadata_value(metadata))
        .extend_pairs(table.parameters().iter().copied());
    Ok(url)
}

pub(super) fn build_raw_request(
    base_url: &Url,
    metadata: bool,
    path: Option<&str>,
    query: &[(Box<str>, Box<str>)],
) -> Result<(Box<str>, Url), MoexError> {
    let endpoint = normalize_raw_endpoint_path(path)?;
    let mut url = endpoint_url(base_url, &endpoint)?;
    let has_meta = query.iter().any(|(key, _)| key.as_ref() == ISS_META_PARAM);
    {
        let mut pairs = url.query_pairs_mut();
        if !has_meta {
            pairs.append_pair(ISS_META_PARAM, metadata_value(metadata));
        }
        pairs.extend_pairs(
            query
                .iter()
                .map(|(key, value)| (key.as_ref(), value.as_ref())),
        );
    }
    Ok((endpoint, url))
}

pub(super) fn validate_payload(
    endpoint: &str,
    status: reqwest::StatusCode,
    content_type: Option<reqwest::header::HeaderValue>,
    payload: String,
) -> Result<String, MoexError> {
    let content_type = content_type.as_ref().and_then(|value| value.to_str().ok());
    if !status.is_success() {
        return Err(MoexError::HttpStatus {
            endpoint: endpoint.to_owned().into_boxed_str(),
            status,
            content_type: content_type.map(Into::into),
            body_prefix: truncate_prefix(&payload, NON_JSON_BODY_PREFIX_CHARS),
        });
    }
    if !looks_like_json_payload(content_type, &payload) {
        return Err(MoexError::NonJsonPayload {
            endpoint: endpoint.to_owned().into_boxed_str(),
            content_type: content_type.map(Into::into),
            body_prefix: truncate_prefix(&payload, NON_JSON_BODY_PREFIX_CHARS),
        });
    }
    Ok(payload)
}

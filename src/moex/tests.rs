#![cfg(feature = "blocking")]

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use chrono::{NaiveDate, NaiveDateTime};
use reqwest::{
    StatusCode, Url,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};

use crate::models::{BoardId, CandleQuery, EngineName, MarketName, Pagination, Security};
#[cfg(feature = "history")]
use crate::models::{HistoryDates, HistoryQuery};

#[cfg(feature = "history")]
use super::client::append_history_query_to_url;
#[cfg(feature = "history")]
use super::client::optional_single_history_dates;
use super::client::{
    append_candle_query_to_url, append_pagination_to_url, looks_like_json_payload,
    normalize_raw_endpoint_path, optional_single_security, truncate_prefix,
};
use super::constants::*;
use super::*;

fn d(input: &str) -> NaiveDate {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").unwrap()
}

fn dt(input: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap()
}

#[test]
fn raw_endpoint_builder_accepts_typed_endpoint() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let board = BoardId::try_from("TQBR").expect("valid board");

    let _request = client
        .raw_endpoint(IssEndpoint::Securities {
            engine: &engine,
            market: &market,
            board: &board,
        })
        .columns("securities", "SECID,LOTSIZE");
}

#[test]
fn optional_single_security_accepts_empty_and_single_row() {
    let empty = optional_single_security("securities/SBER.json", Vec::new())
        .expect("empty securities table is valid");
    assert!(empty.is_none());

    let single_row = vec![
        Security::try_new(
            "SBER".to_owned(),
            "Сбербанк".to_owned(),
            "ПАО Сбербанк".to_owned(),
            "A".to_owned(),
        )
        .expect("valid security"),
    ];
    let single = optional_single_security("securities/SBER.json", single_row)
        .expect("single row must be accepted");
    assert_eq!(
        single
            .expect("single row must produce security")
            .secid()
            .as_str(),
        "SBER"
    );
}

#[test]
fn optional_single_security_rejects_multiple_rows() {
    let rows = vec![
        Security::try_new(
            "SBER".to_owned(),
            "Сбербанк".to_owned(),
            "ПАО Сбербанк".to_owned(),
            "A".to_owned(),
        )
        .expect("valid security"),
        Security::try_new(
            "GAZP".to_owned(),
            "Газпром".to_owned(),
            "ПАО Газпром".to_owned(),
            "A".to_owned(),
        )
        .expect("valid security"),
    ];

    let err = optional_single_security("securities/SBER.json", rows)
        .expect_err("multiple rows must be rejected");
    match err {
        MoexError::UnexpectedSecurityRows {
            endpoint,
            row_count,
        } => {
            assert_eq!(endpoint.as_ref(), "securities/SBER.json");
            assert_eq!(row_count, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[cfg(feature = "history")]
#[test]
fn optional_single_history_dates_accepts_empty_and_single_row() {
    let empty = optional_single_history_dates("history/.../dates.json", Vec::new())
        .expect("empty dates table is valid");
    assert!(empty.is_none());

    let single_row =
        vec![HistoryDates::try_new(d("2013-03-25"), d("2026-03-06")).expect("valid dates")];
    let single = optional_single_history_dates("history/.../dates.json", single_row)
        .expect("single row must be accepted");
    assert_eq!(
        single.expect("single row must produce dates").from(),
        d("2013-03-25")
    );
}

#[cfg(feature = "history")]
#[test]
fn optional_single_history_dates_rejects_multiple_rows() {
    let rows = vec![
        HistoryDates::try_new(d("2013-03-25"), d("2026-03-06")).expect("valid dates"),
        HistoryDates::try_new(d("2011-01-01"), d("2012-01-01")).expect("valid dates"),
    ];
    let err = optional_single_history_dates("history/.../dates.json", rows)
        .expect_err("multiple rows must be rejected");
    match err {
        MoexError::UnexpectedHistoryDatesRows {
            endpoint,
            row_count,
        } => {
            assert_eq!(endpoint.as_ref(), "history/.../dates.json");
            assert_eq!(row_count, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn append_pagination_to_url_sets_start_and_limit() {
    let mut url = Url::parse("https://example.test/iss/engines.json").expect("valid URL");
    let pagination = Pagination::default()
        .with_start(200)
        .with_limit(NonZeroU32::new(5_000).expect("non-zero limit"));

    append_pagination_to_url(&mut url, pagination);

    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == START_PARAM && v == "200")
    );
    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == LIMIT_PARAM && v == "5000")
    );
}

#[test]
fn append_candle_query_to_url_uses_datetime_format() {
    let mut url = Url::parse("https://example.test/iss/candles.json").expect("valid URL");
    let query = CandleQuery::default()
        .with_from(dt("2026-03-01 10:15:30"))
        .expect("valid from")
        .with_till(dt("2026-03-06 18:45:00"))
        .expect("valid till");

    append_candle_query_to_url(&mut url, query);

    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == FROM_PARAM && v == "2026-03-01 10:15:30")
    );
    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == TILL_PARAM && v == "2026-03-06 18:45:00")
    );
}

#[cfg(feature = "history")]
#[test]
fn append_history_query_to_url_uses_date_format() {
    let mut url = Url::parse("https://example.test/iss/history.json").expect("valid URL");
    let query = HistoryQuery::default()
        .with_from(d("2026-03-01"))
        .expect("valid from")
        .with_till(d("2026-03-06"))
        .expect("valid till");

    append_history_query_to_url(&mut url, query);

    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == FROM_PARAM && v == "2026-03-01")
    );
    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == TILL_PARAM && v == "2026-03-06")
    );
}

#[test]
fn append_pagination_to_url_skips_absent_values() {
    let mut url = Url::parse("https://example.test/iss/engines.json").expect("valid URL");

    append_pagination_to_url(&mut url, Pagination::default());

    assert_eq!(url.query(), None);
}

#[test]
fn looks_like_json_payload_accepts_json_content_type() {
    assert!(looks_like_json_payload(
        Some("application/json; charset=utf-8"),
        "<html>not-json</html>"
    ));
}

#[test]
fn looks_like_json_payload_accepts_json_body_without_header() {
    assert!(looks_like_json_payload(None, "  {\"ok\":true}"));
    assert!(looks_like_json_payload(Some("text/plain"), "\n [1,2,3]"));
}

#[test]
fn looks_like_json_payload_rejects_html_payload() {
    assert!(!looks_like_json_payload(
        Some("text/html; charset=utf-8"),
        "<!DOCTYPE html><html></html>"
    ));
}

#[test]
fn truncate_prefix_limits_length() {
    let text = "abcdef";
    assert_eq!(truncate_prefix(text, 4).as_ref(), "abcd");
    assert_eq!(truncate_prefix(text, 10).as_ref(), "abcdef");
}

#[test]
fn builder_constructs_client() {
    let client = BlockingMoexClient::builder()
        .metadata(true)
        .build()
        .expect("valid default builder");
    assert_eq!(client.rate_limit(), None);
}

#[test]
fn constructors_without_semantic_bool_build_clients() {
    let _default = BlockingMoexClient::new().expect("default constructor must work");
    let _with_metadata =
        BlockingMoexClient::new_with_metadata().expect("metadata constructor must work");

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_client =
        BlockingMoexClient::with_client(blocking_http).expect("constructor with client");

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_client_metadata = BlockingMoexClient::with_client_with_metadata(blocking_http)
        .expect("constructor with client and metadata");

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_base_url = BlockingMoexClient::with_base_url(
        blocking_http,
        Url::parse(BASE_URL).expect("valid base url"),
    );

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_base_url_metadata = BlockingMoexClient::with_base_url_with_metadata(
        blocking_http,
        Url::parse(BASE_URL).expect("valid base url"),
    );
}

#[test]
fn builder_rate_limit_is_propagated_to_client() {
    let limit = RateLimit::every(Duration::from_millis(250));
    let client = BlockingMoexClient::builder()
        .rate_limit(limit)
        .build()
        .expect("builder with rate limit must construct client");

    assert_eq!(client.rate_limit(), Some(limit));
}

#[test]
fn builder_user_agent_from_crate_constructs_client() {
    let _client = BlockingMoexClient::builder()
        .user_agent_from_crate()
        .build()
        .expect("user-agent from crate must be valid");
}

#[test]
fn builder_proxy_constructs_client() {
    let proxy = reqwest::Proxy::all("http://127.0.0.1:8090").expect("valid proxy URL");
    let _client = BlockingMoexClient::builder()
        .proxy(proxy)
        .build()
        .expect("builder with proxy must construct client");
}

#[test]
fn builder_no_proxy_constructs_client() {
    let _client = BlockingMoexClient::builder()
        .no_proxy()
        .build()
        .expect("builder with no_proxy must construct client");
}

#[cfg(feature = "async")]
#[test]
fn async_builder_constructs_client() {
    let client = AsyncMoexClient::builder()
        .metadata(true)
        .build()
        .expect("valid async default builder");
    assert_eq!(client.rate_limit(), None);
}

#[cfg(feature = "async")]
#[test]
fn async_constructors_without_semantic_bool_build_clients() {
    let _default = AsyncMoexClient::new().expect("default constructor must work");
    let _with_metadata = AsyncMoexClient::new_with_metadata().expect("metadata constructor");

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_client = AsyncMoexClient::with_client(async_http).expect("constructor with client");

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_client_metadata =
        AsyncMoexClient::with_client_with_metadata(async_http).expect("with metadata");

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_base_url =
        AsyncMoexClient::with_base_url(async_http, Url::parse(BASE_URL).expect("valid base url"));

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_base_url_metadata = AsyncMoexClient::with_base_url_with_metadata(
        async_http,
        Url::parse(BASE_URL).expect("valid base url"),
    );
}

#[cfg(feature = "async")]
#[test]
fn async_builder_rate_limit_without_sleep_returns_error() {
    let err = match AsyncMoexClient::builder()
        .rate_limit(RateLimit::every(Duration::from_millis(250)))
        .build()
    {
        Ok(_) => panic!("async rate limit without sleep must fail on build"),
        Err(err) => err,
    };

    assert!(matches!(err, MoexError::MissingAsyncRateLimitSleep));
}

#[cfg(feature = "async")]
#[test]
fn async_builder_rate_limit_with_sleep_constructs_client() {
    let limit = RateLimit::every(Duration::from_millis(250));
    let client = AsyncMoexClient::builder()
        .rate_limit(limit)
        .rate_limit_sleep(|_delay| std::future::ready(()))
        .build()
        .expect("async builder with rate limit sleep must construct client");

    assert_eq!(client.rate_limit(), Some(limit));
}

#[cfg(feature = "async")]
#[test]
fn async_builder_user_agent_from_crate_constructs_client() {
    let _client = AsyncMoexClient::builder()
        .user_agent_from_crate()
        .build()
        .expect("async user-agent from crate must be valid");
}

#[cfg(feature = "async")]
#[test]
fn async_builder_proxy_constructs_client() {
    let proxy = reqwest::Proxy::all("http://127.0.0.1:8090").expect("valid proxy URL");
    let _client = AsyncMoexClient::builder()
        .proxy(proxy)
        .build()
        .expect("async builder with proxy must construct client");
}

#[cfg(feature = "async")]
#[test]
fn async_builder_no_proxy_constructs_client() {
    let _client = AsyncMoexClient::builder()
        .no_proxy()
        .build()
        .expect("async builder with no_proxy must construct client");
}

#[test]
fn builder_propagates_http_client_build_errors() {
    let err = match BlockingMoexClient::builder()
        .user_agent("invalid\nagent")
        .build()
    {
        Ok(_) => panic!("invalid user-agent must fail on build"),
        Err(err) => err,
    };

    assert!(
        matches!(err, MoexError::BuildHttpClient { .. }),
        "unexpected error variant: {err:?}"
    );
}

#[cfg(feature = "async")]
#[test]
fn async_builder_propagates_http_client_build_errors() {
    let err = match AsyncMoexClient::builder()
        .user_agent("invalid\nagent")
        .build()
    {
        Ok(_) => panic!("invalid user-agent must fail on async build"),
        Err(err) => err,
    };

    assert!(
        matches!(err, MoexError::BuildHttpClient { .. }),
        "unexpected error variant: {err:?}"
    );
}

#[test]
fn owned_scopes_accept_try_into_inputs_and_shortcuts() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");
    let page_limit = NonZeroU32::new(1000).expect("non-zero page limit");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let market_scope = client
        .stock()
        .expect("valid stock shortcut")
        .shares()
        .expect("valid shares shortcut");

    let board_scope = market_scope.clone().board(&board).expect("valid board");
    assert_eq!(board_scope.engine().as_str(), "stock");
    assert_eq!(board_scope.market().as_str(), "shares");
    assert_eq!(board_scope.board().as_str(), "TQBR");

    let market_security_scope = market_scope
        .security("SBER")
        .expect("valid market security");
    assert_eq!(market_security_scope.security().as_str(), "SBER");

    let security_scope = board_scope
        .clone()
        .security("SBER")
        .expect("valid security");
    assert_eq!(security_scope.security().as_str(), "SBER");

    let _securities_pages = board_scope.securities_pages(page_limit);
    let _trades_pages = security_scope.trades_pages(page_limit);

    let index_scope = client.index("IMOEX").expect("valid indexid");
    assert_eq!(index_scope.indexid().as_str(), "IMOEX");
    let _analytics_pages = index_scope.analytics_pages(page_limit);

    let security_resource_scope = client.security("SBER").expect("valid secid");
    assert_eq!(security_resource_scope.secid().as_str(), "SBER");
}

#[test]
fn owned_scopes_fail_fast_on_invalid_identifiers() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");

    let invalid_engine = client.engine("");
    assert!(
        matches!(
            invalid_engine,
            Err(crate::models::ParseEngineNameError::Empty)
        ),
        "unexpected engine parse result"
    );

    let invalid_board = client
        .stock()
        .expect("valid stock shortcut")
        .shares()
        .expect("valid shares shortcut")
        .board(" ");
    assert!(
        matches!(invalid_board, Err(crate::models::ParseBoardIdError::Empty)),
        "unexpected board parse result"
    );

    let invalid_security = client.security(" ");
    assert!(
        matches!(invalid_security, Err(crate::models::ParseSecIdError::Empty)),
        "unexpected security parse result"
    );
}

#[cfg(feature = "async")]
#[test]
fn async_owned_scopes_accept_try_into_inputs_and_shortcuts() {
    let client = AsyncMoexClient::builder()
        .build()
        .expect("valid async default builder");
    let page_limit = NonZeroU32::new(1000).expect("non-zero page limit");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let market_scope = client
        .stock()
        .expect("valid stock shortcut")
        .shares()
        .expect("valid shares shortcut");

    let board_scope = market_scope.clone().board(&board).expect("valid board");
    assert_eq!(board_scope.engine().as_str(), "stock");
    assert_eq!(board_scope.market().as_str(), "shares");
    assert_eq!(board_scope.board().as_str(), "TQBR");

    let market_security_scope = market_scope
        .security("SBER")
        .expect("valid market security");
    assert_eq!(market_security_scope.security().as_str(), "SBER");

    let security_scope = board_scope
        .clone()
        .security("SBER")
        .expect("valid security");
    assert_eq!(security_scope.security().as_str(), "SBER");

    let _securities_pages = board_scope.securities_pages(page_limit);
    let _trades_pages = security_scope.trades_pages(page_limit);

    let index_scope = client.index("IMOEX").expect("valid indexid");
    assert_eq!(index_scope.indexid().as_str(), "IMOEX");
    let _analytics_pages = index_scope.analytics_pages(page_limit);

    let security_resource_scope = client.security("SBER").expect("valid secid");
    assert_eq!(security_resource_scope.secid().as_str(), "SBER");
}

#[test]
fn normalize_raw_endpoint_path_supports_common_forms() {
    assert_eq!(
        normalize_raw_endpoint_path(Some("engines"))
            .expect("must normalize")
            .as_ref(),
        "engines.json"
    );
    assert_eq!(
        normalize_raw_endpoint_path(Some("engines.json"))
            .expect("must keep explicit extension")
            .as_ref(),
        "engines.json"
    );
    assert_eq!(
        normalize_raw_endpoint_path(Some("/iss/engines"))
            .expect("must trim /iss/ prefix")
            .as_ref(),
        "engines.json"
    );
}

#[test]
fn normalize_raw_endpoint_path_rejects_missing_or_invalid_path() {
    let missing = normalize_raw_endpoint_path(None).expect_err("path is required");
    assert!(matches!(missing, MoexError::MissingRawPath));

    let empty = normalize_raw_endpoint_path(Some("  ")).expect_err("empty path must fail");
    assert!(matches!(empty, MoexError::InvalidRawPath { .. }));

    let with_query = normalize_raw_endpoint_path(Some("engines?iss.meta=on"))
        .expect_err("query string in path must fail");
    assert!(matches!(with_query, MoexError::InvalidRawPath { .. }));
}

#[test]
fn raw_builder_requires_path_before_send() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");

    let err = client
        .raw()
        .send_payload()
        .expect_err("raw request without path must fail");
    assert!(matches!(err, MoexError::MissingRawPath));
}

#[test]
fn iss_request_options_and_raw_response_helpers_work() {
    let options = IssRequestOptions::new()
        .metadata(IssToggle::On)
        .data(IssToggle::Off)
        .version(IssToggle::On)
        .json("extended");
    assert_eq!(options.metadata_value(), Some(IssToggle::On));
    assert_eq!(options.data_value(), Some(IssToggle::Off));
    assert_eq!(options.version_value(), Some(IssToggle::On));
    assert_eq!(options.json_value(), Some("extended"));

    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");
    let _raw = client
        .raw()
        .path("engines")
        .options(options)
        .metadata(IssToggle::On)
        .data(IssToggle::On)
        .version(IssToggle::Off)
        .json("extended");

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let response = RawIssResponse::new(StatusCode::OK, headers.clone(), "{\"ok\":true}".into());
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .expect("content-type header")
            .to_str()
            .expect("header to str"),
        "application/json"
    );
    assert_eq!(response.body(), "{\"ok\":true}");

    let (status, returned_headers, body) = response.into_parts();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        returned_headers
            .get(CONTENT_TYPE)
            .expect("content-type header")
            .to_str()
            .expect("header to str"),
        "application/json"
    );
    assert_eq!(body, "{\"ok\":true}");
}

#[test]
fn moex_error_helpers_detect_retryable_status_and_body_prefix() {
    let err = MoexError::HttpStatus {
        endpoint: "engines.json".into(),
        status: StatusCode::TOO_MANY_REQUESTS,
        content_type: Some("application/json".into()),
        body_prefix: "{\"error\":\"rate limit\"}".into(),
    };

    assert!(err.is_retryable());
    assert_eq!(err.status_code(), Some(StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(
        err.response_body_prefix(),
        Some("{\"error\":\"rate limit\"}")
    );
}

#[test]
fn moex_error_helpers_mark_non_retryable_client_status() {
    let err = MoexError::HttpStatus {
        endpoint: "engines.json".into(),
        status: StatusCode::BAD_REQUEST,
        content_type: Some("application/json".into()),
        body_prefix: "{\"error\":\"bad request\"}".into(),
    };

    assert!(!err.is_retryable());
    assert_eq!(err.status_code(), Some(StatusCode::BAD_REQUEST));
}

#[test]
fn moex_error_helpers_return_body_prefix_for_non_json_payload() {
    let err = MoexError::NonJsonPayload {
        endpoint: "engines.json".into(),
        content_type: Some("text/html".into()),
        body_prefix: "<!doctype html>".into(),
    };

    assert_eq!(err.response_body_prefix(), Some("<!doctype html>"));
    assert_eq!(err.status_code(), None);
    assert!(!err.is_retryable());
}

#[test]
fn with_retry_retries_retryable_errors_until_success() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(3).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let value = with_retry(policy, || {
        attempts += 1;
        if attempts < 3 {
            return Err(MoexError::HttpStatus {
                endpoint: "engines.json".into(),
                status: StatusCode::TOO_MANY_REQUESTS,
                content_type: Some("application/json".into()),
                body_prefix: "{\"error\":\"rate limit\"}".into(),
            });
        }
        Ok(42_u32)
    })
    .expect("third attempt must succeed");

    assert_eq!(value, 42_u32);
    assert_eq!(attempts, 3);
}

#[test]
fn retry_policy_default_matches_example_values() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_attempts().get(), 3);
    assert_eq!(policy.delay(), Duration::from_millis(400));
}

#[test]
fn with_retry_stops_on_non_retryable_error_without_extra_attempts() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(5).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let err = with_retry::<(), _>(policy, || {
        attempts += 1;
        Err(MoexError::HttpStatus {
            endpoint: "engines.json".into(),
            status: StatusCode::BAD_REQUEST,
            content_type: Some("application/json".into()),
            body_prefix: "{\"error\":\"bad request\"}".into(),
        })
    })
    .expect_err("non-retryable error must be returned immediately");

    assert!(matches!(
        err,
        MoexError::HttpStatus {
            status: StatusCode::BAD_REQUEST,
            ..
        }
    ));
    assert_eq!(attempts, 1);
}

#[test]
fn with_retry_stops_after_max_attempts_on_retryable_error() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(2).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let err = with_retry::<(), _>(policy, || {
        attempts += 1;
        Err(MoexError::HttpStatus {
            endpoint: "engines.json".into(),
            status: StatusCode::SERVICE_UNAVAILABLE,
            content_type: Some("application/json".into()),
            body_prefix: "{\"error\":\"temporary unavailable\"}".into(),
        })
    })
    .expect_err("retryable error must stop after max attempts");

    assert!(matches!(
        err,
        MoexError::HttpStatus {
            status: StatusCode::SERVICE_UNAVAILABLE,
            ..
        }
    ));
    assert_eq!(attempts, 2);
}

#[test]
fn rate_limit_per_second_rounds_up_interval() {
    let limit = RateLimit::per_second(NonZeroU32::new(3).expect("non-zero requests per second"));
    assert_eq!(limit.min_interval(), Duration::from_nanos(333_333_334));
}

#[test]
fn rate_limiter_reserve_delay_at_tracks_slots() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_millis(100)));
    let start = Instant::now();

    assert_eq!(limiter.reserve_delay_at(start), Duration::ZERO);
    assert_eq!(limiter.reserve_delay_at(start), Duration::from_millis(100));
    assert_eq!(
        limiter.reserve_delay_at(start + Duration::from_millis(250)),
        Duration::ZERO
    );
    assert_eq!(
        limiter.reserve_delay_at(start + Duration::from_millis(300)),
        Duration::from_millis(50)
    );
}

#[test]
fn with_rate_limit_executes_action_and_returns_value() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::ZERO));
    let value = with_rate_limit(&mut limiter, || 42_u32);
    assert_eq!(value, 42_u32);
}

#[cfg(feature = "async")]
fn block_on_immediate<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut future = pin!(future);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("test future unexpectedly returned Poll::Pending"),
    }
}

#[cfg(feature = "async")]
#[test]
fn with_rate_limit_async_skips_sleep_on_first_slot() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_secs(1)));
    let mut sleep_calls = 0_u32;

    let value = block_on_immediate(with_rate_limit_async(
        &mut limiter,
        || std::future::ready(7_u32),
        |_delay| {
            sleep_calls += 1;
            std::future::ready(())
        },
    ));

    assert_eq!(value, 7_u32);
    assert_eq!(sleep_calls, 0);
}

#[cfg(feature = "async")]
#[test]
fn with_rate_limit_async_uses_injected_sleep_when_delay_is_reserved() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_secs(1)));
    limiter.reserve_delay_at(Instant::now());
    let mut slept = Vec::new();

    let value = block_on_immediate(with_rate_limit_async(
        &mut limiter,
        || std::future::ready(9_u32),
        |delay| {
            slept.push(delay);
            std::future::ready(())
        },
    ));

    assert_eq!(value, 9_u32);
    assert_eq!(slept.len(), 1);
    assert!(slept[0] > Duration::ZERO);
    assert!(slept[0] <= Duration::from_secs(1));
}

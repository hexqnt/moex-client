use std::num::NonZeroU32;

#[cfg(feature = "history")]
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use reqwest::{
    StatusCode, Url,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};

#[cfg(feature = "history")]
use crate::models::HistoryQuery;
use crate::models::{BoardId, CandleQuery, EngineName, MarketName, Pagination};

#[cfg(feature = "history")]
use crate::moex::client::append_history_query_to_url;
use crate::moex::client::{
    append_candle_query_to_url, append_pagination_to_url, looks_like_json_payload,
    normalize_raw_endpoint_path, truncate_prefix,
};
use crate::moex::constants::*;
use crate::moex::*;

#[cfg(feature = "history")]
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

use super::*;
use reqwest::StatusCode;

fn tracker(limit: u32) -> PaginationTracker<u32> {
    PaginationTracker::new("test.json", NonZeroU32::new(limit).expect("non-zero limit"))
}

#[test]
fn pagination_yields_full_and_short_pages_and_stops() {
    let mut tracker = tracker(2);
    assert_eq!(
        tracker.next_page_request().expect("first page").start,
        Some(0)
    );
    assert_eq!(
        tracker
            .accept_page(vec![1, 2], |key| *key)
            .expect("full page"),
        Some(vec![1, 2])
    );
    assert_eq!(
        tracker.next_page_request().expect("second page").start,
        Some(2)
    );
    assert_eq!(
        tracker
            .accept_page(vec![3], |key| *key)
            .expect("short page"),
        Some(vec![3])
    );
    assert_eq!(tracker.next_page_request(), None);
}

#[test]
fn pagination_empty_page_finishes_without_extracting_a_key() {
    let mut tracker = tracker(2);
    assert_eq!(
        tracker
            .accept_page(Vec::<u32>::new(), |_| panic!("empty page has no key"))
            .expect("empty page"),
        None
    );
    assert_eq!(tracker.next_page_request(), None);
}

#[test]
fn pagination_rejects_repeated_first_key_without_advancing() {
    let mut tracker = tracker(2);
    tracker
        .accept_page(vec![1, 2], |key| *key)
        .expect("first page");
    let error = tracker
        .accept_page(vec![1, 3], |key| *key)
        .expect_err("repeated first key");
    assert!(
        matches!(error, MoexError::PaginationStuck { endpoint, start: 2, limit: 2 } if endpoint.as_ref() == "test.json")
    );
    assert_eq!(
        tracker.next_page_request().expect("retry page").start,
        Some(2)
    );
    assert_eq!(
        tracker
            .accept_page(vec![3], |key| *key)
            .expect("replacement page"),
        Some(vec![3])
    );
}

#[test]
fn pagination_reports_overflow_instead_of_wrapping() {
    let mut tracker = tracker(2);
    tracker.start = u32::MAX - 1;
    let error = tracker
        .accept_page(vec![1, 2], |key| *key)
        .expect_err("overflow");
    assert!(
        matches!(error, MoexError::PaginationOverflow { endpoint, start, limit: 2 } if endpoint.as_ref() == "test.json" && start == u32::MAX - 1)
    );
    assert_eq!(
        tracker.next_page_request().expect("same offset").start,
        Some(u32::MAX - 1)
    );
}

#[test]
fn table_request_preserves_query_order_for_multiple_tables_and_pagination() {
    let base = Url::parse("https://example.test/custom/iss/").expect("valid URL");
    let mut url =
        table_url(&base, true, "securities.json", TableSelection::Snapshots).expect("table URL");
    append_pagination_to_url(
        &mut url,
        Pagination::default()
            .with_start(0)
            .with_limit(NonZeroU32::new(2).expect("non-zero limit")),
    );
    assert_eq!(url.path(), "/custom/iss/securities.json");
    assert_eq!(
        url.query_pairs().collect::<Vec<_>>(),
        [
            ("iss.meta".into(), "on".into()),
            ("iss.only".into(), "securities,marketdata".into()),
            ("securities.columns".into(), "SECID,LOTSIZE".into()),
            ("marketdata.columns".into(), "SECID,LAST".into()),
            ("start".into(), "0".into()),
            ("limit".into(), "2".into()),
        ]
    );
}

#[test]
fn raw_request_preserves_explicit_metadata_and_repeated_parameters() {
    let base = Url::parse("https://example.test/iss/").expect("valid URL");
    let query = [
        ("iss.only".into(), "securities".into()),
        ("iss.meta".into(), "on".into()),
        ("iss.meta".into(), "off".into()),
    ];
    let (endpoint, url) =
        build_raw_request(&base, false, Some(" /iss/securities "), &query).expect("raw URL");
    assert_eq!(endpoint.as_ref(), "securities.json");
    assert_eq!(
        url.query(),
        Some("iss.only=securities&iss.meta=on&iss.meta=off")
    );
    let (_, url) = build_raw_request(&base, true, Some("engines"), &[]).expect("default metadata");
    assert_eq!(url.query(), Some("iss.meta=on"));
}

#[test]
fn payload_http_error_takes_precedence_over_content_detection() {
    let body = "я".repeat(NON_JSON_BODY_PREFIX_CHARS + 1);
    let error = validate_payload(
        "engines.json",
        StatusCode::BAD_GATEWAY,
        Some("text/html".into()),
        body,
    )
    .expect_err("HTTP error");
    assert!(
        matches!(error, MoexError::HttpStatus { endpoint, status: StatusCode::BAD_GATEWAY, content_type: Some(content_type), body_prefix } if endpoint.as_ref() == "engines.json" && content_type.as_ref() == "text/html" && body_prefix.chars().count() == NON_JSON_BODY_PREFIX_CHARS)
    );
}

#[test]
fn payload_detection_keeps_header_and_body_fallback_contracts() {
    assert_eq!(
        validate_payload(
            "test.json",
            StatusCode::OK,
            Some("APPLICATION/JSON".into()),
            "text".into()
        )
        .expect("JSON content type"),
        "text"
    );
    assert_eq!(
        validate_payload("test.json", StatusCode::OK, None, " \n[]".into()).expect("JSON body"),
        " \n[]"
    );
    assert!(
        matches!(validate_payload("test.json", StatusCode::OK, None, "<html>".into()), Err(MoexError::NonJsonPayload { endpoint, content_type: None, body_prefix }) if endpoint.as_ref() == "test.json" && body_prefix.as_ref() == "<html>")
    );
}

#[cfg(feature = "blocking")]
#[test]
fn blocking_finished_paginator_never_fetches_another_page() {
    let client = BlockingMoexClient::new().expect("client");
    let mut pages = client.global_securities_pages(NonZeroU32::new(2).expect("non-zero limit"));
    pages.pagination.finished = true;
    assert_eq!(pages.next_page().expect("finished paginator"), None);
    assert!(pages.try_collect().expect("empty collection").is_empty());
}

#[cfg(feature = "async")]
fn immediate<F: std::future::Future>(future: F) -> F::Output {
    use std::task::{Context, Poll, Waker};
    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("future unexpectedly requires a runtime"),
    }
}

#[cfg(feature = "async")]
#[test]
fn async_finished_paginator_and_missing_raw_path_need_no_runtime() {
    let client = AsyncMoexClient::new().expect("client");
    let mut pages = client.global_securities_pages(NonZeroU32::new(2).expect("non-zero limit"));
    pages.pagination.finished = true;
    assert_eq!(
        immediate(pages.next_page()).expect("finished paginator"),
        None
    );
    assert!(immediate(pages.all()).expect("empty collection").is_empty());
    assert!(matches!(
        immediate(client.raw().send_response()),
        Err(MoexError::MissingRawPath)
    ));
    assert!(matches!(
        immediate(client.raw().send_payload()),
        Err(MoexError::MissingRawPath)
    ));
}

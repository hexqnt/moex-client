#[cfg(feature = "async")]
use crate::moex::tests::immediate;

use super::*;

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
fn pagination_repeated_key_is_rejected_even_on_a_short_page() {
    let mut tracker = tracker(2);
    tracker
        .accept_page(vec![1, 2], |key| *key)
        .expect("first page");
    assert!(matches!(
        tracker.accept_page(vec![1], |key| *key),
        Err(MoexError::PaginationStuck {
            start: 2,
            limit: 2,
            ..
        })
    ));
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
    assert_eq!(tracker.next_page_request(), None);
}

#[test]
fn pagination_oversized_page_advances_by_requested_limit() {
    let mut tracker = tracker(2);
    assert_eq!(
        tracker
            .accept_page(vec![1, 2, 3], |key| *key)
            .expect("oversized page"),
        Some(vec![1, 2, 3])
    );
    assert_eq!(
        tracker.next_page_request().expect("next page").start,
        Some(2)
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
    assert!(matches!(
        tracker.accept_page(vec![1, 2], |key| *key),
        Err(MoexError::PaginationStuck { start, limit: 2, .. }) if start == u32::MAX - 1
    ));
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

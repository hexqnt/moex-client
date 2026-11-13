mod candles;
mod catalog;
#[cfg(feature = "history")]
mod history;
mod identifiers;
mod indexes;
mod news;
mod selectors;
mod trading;
mod wire;

use std::num::NonZeroU32;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::*;

fn d(input: &str) -> NaiveDate {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").unwrap()
}

fn dt(input: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap()
}

fn t(input: &str) -> NaiveTime {
    NaiveTime::parse_from_str(input, "%H:%M:%S").unwrap()
}

#[test]
fn pagination_builder_sets_start_and_limit() {
    let pagination = Pagination::default()
        .with_start(150)
        .with_limit(NonZeroU32::new(5_000).expect("non-zero limit"));

    assert_eq!(pagination.start, Some(150));
    assert_eq!(
        pagination.limit,
        Some(NonZeroU32::new(5_000).expect("non-zero limit"))
    );
}

#[test]
fn page_request_helpers_construct_expected_variants() {
    let first = PageRequest::first_page();
    assert_eq!(first, PageRequest::FirstPage);

    let pagination = Pagination::default()
        .with_start(100)
        .with_limit(NonZeroU32::new(250).expect("non-zero limit"));
    let page = PageRequest::page(pagination);
    assert_eq!(page, PageRequest::Page(pagination));

    let all = PageRequest::all(NonZeroU32::new(1_000).expect("non-zero limit"));
    assert_eq!(
        all,
        PageRequest::All {
            page_limit: NonZeroU32::new(1_000).expect("non-zero limit")
        }
    );
}

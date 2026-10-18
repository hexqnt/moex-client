use super::*;

#[test]
fn news_preserves_field_error_priority_and_positive_id_range() {
    let at = dt("2026-03-06 10:00:00");
    for (id, tag, expected_news, expected_event) in [
        (
            0,
            "",
            ParseSiteNewsError::NonPositiveId(0),
            ParseEventError::NonPositiveId(0),
        ),
        (
            1,
            "",
            ParseSiteNewsError::EmptyTag,
            ParseEventError::EmptyTag,
        ),
        (
            1,
            "site",
            ParseSiteNewsError::EmptyTitle,
            ParseEventError::EmptyTitle,
        ),
    ] {
        assert_eq!(
            SiteNews::try_new(id, tag.into(), "".into(), at, at),
            Err(expected_news)
        );
        assert_eq!(
            Event::try_new(id, tag.into(), "".into(), None, at),
            Err(expected_event)
        );
    }
    assert_eq!(
        SiteNews::try_new(i64::MAX, "site".into(), "Title".into(), at, at)
            .expect("largest wire news id")
            .id(),
        i64::MAX as u64
    );
    assert_eq!(
        Event::try_new(i64::MAX, "site".into(), "Title".into(), None, at)
            .expect("largest wire event id")
            .id(),
        i64::MAX as u64
    );
}

#[cfg(feature = "news")]
#[test]
fn parses_sitenews_row() {
    let row: SiteNewsRow = serde_json::from_str(
        r#"[98236,"site","О дополнительных условиях проведения торгов","2026-03-06 19:08:57","2026-03-06 19:08:57"]"#,
    )
    .expect("valid sitenews row");
    let news = SiteNews::try_from(row).expect("valid sitenews");
    assert_eq!(news.id(), 98_236);
    assert_eq!(news.tag(), "site");
    assert_eq!(news.title(), "О дополнительных условиях проведения торгов");
    assert_eq!(news.published_at(), dt("2026-03-06 19:08:57"));
}

#[cfg(feature = "news")]
#[test]
fn rejects_sitenews_with_empty_title() {
    let row: SiteNewsRow =
        serde_json::from_str(r#"[98236,"site","  ","2026-03-06 19:08:57","2026-03-06 19:08:57"]"#)
            .expect("valid sitenews row");
    let err = SiteNews::try_from(row).expect_err("must reject empty sitenews title");
    assert!(matches!(err, ParseSiteNewsError::EmptyTitle));
}

#[cfg(feature = "news")]
#[test]
fn parses_event_row_with_optional_from() {
    let row: EventRow = serde_json::from_str(
        r#"[77,"site","Технические работы","2026-03-07 10:00:00","2026-03-06 22:00:00"]"#,
    )
    .expect("valid events row");
    let event = Event::try_from(row).expect("valid event");
    assert_eq!(event.id(), 77);
    assert_eq!(event.tag(), "site");
    assert_eq!(event.from(), Some(dt("2026-03-07 10:00:00")));
    assert_eq!(event.modified_at(), dt("2026-03-06 22:00:00"));
}

#[cfg(feature = "news")]
#[test]
fn parses_event_row_with_null_from() {
    let row: EventRow =
        serde_json::from_str(r#"[78,"site","Событие без from",null,"2026-03-06 22:00:00"]"#)
            .expect("valid events row");
    let event = Event::try_from(row).expect("valid event");
    assert_eq!(event.id(), 78);
    assert_eq!(event.from(), None);
}

#[cfg(feature = "news")]
#[test]
fn rejects_event_with_empty_tag() {
    let row: EventRow =
        serde_json::from_str(r#"[77,"  ","Технические работы",null,"2026-03-06 22:00:00"]"#)
            .expect("valid events row");
    let err = Event::try_from(row).expect_err("must reject empty event tag");
    assert!(matches!(err, ParseEventError::EmptyTag));
}

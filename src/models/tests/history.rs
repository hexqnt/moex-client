use super::*;

#[test]
fn parses_history_dates_row() {
    let row: HistoryDatesRow =
        serde_json::from_str(r#"["2013-03-25","2026-03-06"]"#).expect("valid history dates row");
    let dates = HistoryDates::try_from(row).expect("valid history dates");
    assert_eq!(dates.from(), d("2013-03-25"));
    assert_eq!(dates.till(), d("2026-03-06"));
}

#[test]
fn rejects_invalid_history_dates_range() {
    let err = HistoryDates::try_new(d("2026-03-07"), d("2026-03-06"))
        .expect_err("must reject invalid history dates range");
    assert!(matches!(
        err,
        ParseHistoryDatesError::InvalidDateRange { .. }
    ));
}

#[test]
fn parses_history_row() {
    let row: HistoryRow = serde_json::from_str(
        r#"{
            "BOARDID":"TQBR",
            "TRADEDATE":"2026-03-06",
            "SECID":"SBER",
            "NUMTRADES":120345,
            "VALUE":123456789.5,
            "OPEN":314.0,
            "LOW":310.0,
            "HIGH":315.2,
            "CLOSE":314.8,
            "VOLUME":3900000,
            "DURATION":1733.5,
            "YIELD":15.49
        }"#,
    )
    .expect("valid history row");
    let history = HistoryRecord::try_from(row).expect("valid history");
    assert_eq!(history.boardid().as_str(), "TQBR");
    assert_eq!(history.tradedate(), d("2026-03-06"));
    assert_eq!(history.secid().as_str(), "SBER");
    assert_eq!(history.numtrades(), Some(120_345));
    assert_eq!(history.close(), Some(314.8));
    assert_eq!(history.volume(), Some(3_900_000));
    assert_eq!(history.duration_days(), Some(1_733.5));
    assert_eq!(history.yield_percent(), Some(15.49));
}

#[test]
fn rejects_negative_history_volume() {
    let row: HistoryRow = serde_json::from_str(
        r#"{
            "BOARDID":"TQBR",
            "TRADEDATE":"2026-03-06",
            "SECID":"SBER",
            "VOLUME":-1
        }"#,
    )
    .expect("valid history row");
    let err = HistoryRecord::try_from(row).expect_err("must reject negative history volume");
    assert!(matches!(err, ParseHistoryRecordError::NegativeVolume(-1)));
}

#[test]
fn history_query_rejects_invalid_range() {
    let err = HistoryQuery::try_new(Some(d("2026-03-06")), Some(d("2026-01-01")))
        .expect_err("must reject invalid query range");
    assert!(matches!(
        err,
        ParseHistoryQueryError::InvalidDateRange { .. }
    ));
}

#[test]
fn history_query_builder_keeps_range_valid() {
    let query = HistoryQuery::default()
        .with_from(d("2026-01-01"))
        .expect("valid from")
        .with_till(d("2026-03-06"))
        .expect("valid till");

    assert_eq!(query.from(), Some(d("2026-01-01")));
    assert_eq!(query.till(), Some(d("2026-03-06")));
}

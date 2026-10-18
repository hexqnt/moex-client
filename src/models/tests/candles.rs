use super::*;

#[test]
fn parses_candle_border_row() {
    let row: CandleBorderRow =
        serde_json::from_str(r#"["2011-12-15 10:00:00","2026-03-06 23:49:00",1,57]"#)
            .expect("valid candle border row");
    let border = CandleBorder::try_from(row).expect("valid candle border");
    assert_eq!(border.begin(), dt("2011-12-15 10:00:00"));
    assert_eq!(border.end(), dt("2026-03-06 23:49:00"));
    assert_eq!(border.interval(), CandleInterval::Minute1);
    assert_eq!(border.board_group_id(), 57);
}

#[test]
fn rejects_invalid_candle_border_interval() {
    let err = CandleBorder::try_new(
        dt("2011-12-15 10:00:00"),
        dt("2026-03-06 23:49:00"),
        999,
        57,
    )
    .expect_err("must reject unknown interval");
    assert!(matches!(
        err,
        ParseCandleBorderError::InvalidInterval(ParseCandleIntervalError::InvalidCode(999))
    ));
}

#[test]
fn parses_candle_row() {
    let row: CandleRow = serde_json::from_str(
            r#"["2007-07-20 00:00:00","2007-07-20 23:59:59",109,109.2,111.11,108.01,2211623351.09,20252489]"#,
        )
        .expect("valid candle row");
    let candle = Candle::try_from(row).expect("valid candle");
    assert_eq!(candle.begin(), dt("2007-07-20 00:00:00"));
    assert_eq!(candle.end(), dt("2007-07-20 23:59:59"));
    assert_eq!(candle.close(), Some(109.2));
    assert_eq!(candle.volume(), Some(20_252_489));
}

#[test]
fn rejects_candle_with_invalid_range() {
    let err = Candle::try_new(
        dt("2007-07-20 23:59:59"),
        dt("2007-07-20 00:00:00"),
        CandleOhlcv::new(
            Some(109.0),
            Some(109.2),
            Some(111.11),
            Some(108.01),
            Some(2211623351.09),
            Some(20_252_489),
        ),
    )
    .expect_err("must reject invalid candle range");
    assert!(matches!(err, ParseCandleError::InvalidDateRange { .. }));
}

#[test]
fn rejects_candle_with_negative_volume() {
    let err = Candle::try_new(
        dt("2007-07-20 00:00:00"),
        dt("2007-07-20 23:59:59"),
        CandleOhlcv::new(
            Some(109.0),
            Some(109.2),
            Some(111.11),
            Some(108.01),
            Some(2211623351.09),
            Some(-1),
        ),
    )
    .expect_err("must reject negative volume");
    assert!(matches!(err, ParseCandleError::NegativeVolume(-1)));
}

#[test]
fn candle_query_rejects_invalid_range() {
    let err = CandleQuery::try_new(
        Some(dt("2026-03-06 00:00:00")),
        Some(dt("2026-01-01 00:00:00")),
        None,
    )
    .expect_err("must reject invalid query range");
    assert!(matches!(
        err,
        ParseCandleQueryError::InvalidDateRange { .. }
    ));
}

#[test]
fn candle_query_builder_keeps_range_valid() {
    let query = CandleQuery::default()
        .with_interval(CandleInterval::Day1)
        .with_from(dt("2026-01-01 00:00:00"))
        .expect("valid from")
        .with_till(dt("2026-03-06 23:59:59"))
        .expect("valid till");

    assert_eq!(query.from(), Some(dt("2026-01-01 00:00:00")));
    assert_eq!(query.till(), Some(dt("2026-03-06 23:59:59")));
    assert_eq!(query.interval(), Some(CandleInterval::Day1));
}

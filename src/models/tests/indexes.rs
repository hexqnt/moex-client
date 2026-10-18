use super::*;

#[test]
fn analytics_preserves_field_error_priority() {
    use ParseIndexAnalyticsError as Error;

    let date = d("2026-03-06");
    for (id, ticker, secid, name, weight, expected) in [
        (
            "",
            "",
            "",
            "",
            f64::NAN,
            Error::InvalidIndexId(ParseIndexError::EmptyIndexId),
        ),
        (
            "IMOEX",
            "",
            "",
            "",
            f64::NAN,
            Error::InvalidTicker(ParseSecIdError::Empty),
        ),
        (
            "IMOEX",
            "SBER",
            "",
            "",
            f64::NAN,
            Error::InvalidSecId(ParseSecIdError::Empty),
        ),
        (
            "IMOEX",
            "SBER",
            "SBER",
            "",
            f64::NAN,
            Error::EmptyShortnames,
        ),
        (
            "IMOEX",
            "SBER",
            "SBER",
            "Sber",
            f64::NAN,
            Error::NonFiniteWeight,
        ),
        ("IMOEX", "SBER", "SBER", "Sber", -0.0, Error::NegativeWeight),
        (
            "IMOEX",
            "SBER",
            "SBER",
            "Sber",
            0.0,
            Error::InvalidTradingsession(0),
        ),
    ] {
        let row = IndexAnalyticsRow(
            id.into(),
            date,
            ticker.into(),
            name.into(),
            secid.into(),
            weight,
            0,
            date,
        );
        assert_eq!(IndexAnalytics::try_from(row), Err(expected));
    }
}

#[test]
fn rejects_invalid_range() {
    let err = Index::try_new(
        "IMOEX".to_string(),
        "Индекс МосБиржи".to_string(),
        Some(d("2026-03-05")),
        Some(d("2001-01-03")),
    )
    .expect_err("must reject invalid range");
    assert!(matches!(err, ParseIndexError::InvalidDateRange { .. }));
}

#[test]
fn parses_index_analytics_row() {
    let row: IndexAnalyticsRow = serde_json::from_str(
        r#"["IMOEX","2026-03-06","SBER","Сбербанк","SBER",14.56,3,"2026-03-06"]"#,
    )
    .expect("valid analytics row");
    let analytics = IndexAnalytics::try_from(row).expect("valid analytics");
    assert_eq!(analytics.indexid().as_str(), "IMOEX");
    assert_eq!(analytics.ticker().as_str(), "SBER");
    assert_eq!(analytics.secid().as_str(), "SBER");
    assert_eq!(analytics.shortnames(), "Сбербанк");
    assert_eq!(analytics.weight(), 14.56);
    assert_eq!(analytics.tradingsession(), 3);
    assert_eq!(analytics.tradedate(), d("2026-03-06"));
}

#[test]
fn rejects_negative_index_analytics_weight() {
    let err = IndexAnalytics::try_from(IndexAnalyticsRow(
        "IMOEX".to_string(),
        d("2026-03-06"),
        "SBER".to_string(),
        "Сбербанк".to_string(),
        "SBER".to_string(),
        -0.01,
        3,
        d("2026-03-06"),
    ))
    .expect_err("must reject negative weight");
    assert!(matches!(err, ParseIndexAnalyticsError::NegativeWeight));
}

#[test]
fn rejects_invalid_index_analytics_tradingsession() {
    let err = IndexAnalytics::try_from(IndexAnalyticsRow(
        "IMOEX".to_string(),
        d("2026-03-06"),
        "SBER".to_string(),
        "Сбербанк".to_string(),
        "SBER".to_string(),
        14.56,
        4,
        d("2026-03-06"),
    ))
    .expect_err("must reject unsupported tradingsession");
    assert!(matches!(
        err,
        ParseIndexAnalyticsError::InvalidTradingsession(4)
    ));
}

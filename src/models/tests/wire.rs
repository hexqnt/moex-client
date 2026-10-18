use super::*;

#[test]
fn optional_wire_dates_preserve_null_empty_trim_and_parse_errors() {
    for raw in [
        serde_json::Value::Null,
        serde_json::json!(""),
        serde_json::json!("\u{2003} "),
    ] {
        let row: IndexRow = serde_json::from_value(serde_json::json!(["IMOEX", "Index", raw, raw]))
            .expect("empty optional dates");
        let index = Index::try_from(row).expect("valid index");
        assert_eq!(index.from(), None);
        assert_eq!(index.till(), None);
    }
    let row: IndexRow =
        serde_json::from_str(r#"["IMOEX","Index"," 2026-03-06 ",null]"#).expect("trimmed date");
    assert_eq!(row.2, Some(d("2026-03-06")));
    assert!(serde_json::from_str::<IndexRow>(r#"["IMOEX","Index","2026-02-30",null]"#).is_err());
}

#[test]
fn mandatory_wire_temporal_fields_reject_null_empty_and_invalid_values() {
    for raw in [
        serde_json::Value::Null,
        serde_json::json!(""),
        serde_json::json!("invalid"),
    ] {
        assert!(
            serde_json::from_value::<IndexAnalyticsRow>(serde_json::json!([
                "IMOEX",
                raw,
                "SBER",
                "Sber",
                "SBER",
                1.0,
                1,
                "2026-03-06"
            ]))
            .is_err()
        );
        assert!(
            serde_json::from_value::<CandleRow>(serde_json::json!([
                raw,
                "2026-03-06 10:00:00",
                null,
                null,
                null,
                null,
                null,
                null
            ]))
            .is_err()
        );
        assert!(
            serde_json::from_value::<TradeRow>(serde_json::json!([1, raw, null, null, null]))
                .is_err()
        );
    }
}

#[test]
fn wire_temporal_fields_accept_escaped_and_owned_text() {
    fn from_json_and_value<T: serde::de::DeserializeOwned>(payload: &str) -> [T; 2] {
        [
            serde_json::from_str(payload).expect("valid escaped temporal text"),
            serde_json::from_value(serde_json::from_str(payload).expect("valid JSON value"))
                .expect("valid owned temporal text"),
        ]
    }

    for row in
        from_json_and_value::<IndexRow>(r#"["IMOEX","Index","\u20032026-03-06\u00a0","\u0020"]"#)
    {
        assert_eq!(row.2, Some(d("2026-03-06")));
        assert_eq!(row.3, None);
    }
    for row in from_json_and_value::<IndexAnalyticsRow>(
        r#"["IMOEX","2026-\u00303-06","SBER","Sber","SBER",1,1,"2026-03-\u00306"]"#,
    ) {
        assert_eq!(row.1, d("2026-03-06"));
        assert_eq!(row.7, d("2026-03-06"));
    }
    for row in from_json_and_value::<CandleRow>(
        r#"["2026-03-06\u002010:00:00","2026-03-06 10:00:00",null,null,null,null,null,null]"#,
    ) {
        assert_eq!(row.0, dt("2026-03-06 10:00:00"));
        assert_eq!(row.1, row.0);
    }
    for row in from_json_and_value::<TradeRow>(r#"[1,"10\u003a00:00",null,null,null]"#) {
        assert_eq!(row.1, t("10:00:00"));
    }
    #[cfg(feature = "news")]
    for row in from_json_and_value::<EventRow>(
        r#"[1,"site","Event","2026-03-06\u002010:00:00","2026-03-06\u002010:00:00"]"#,
    ) {
        assert_eq!(row.3, Some(dt("2026-03-06 10:00:00")));
        assert_eq!(row.3, Some(row.4));
    }
    assert!(
        serde_json::from_str::<IndexRow>(r#"["IMOEX","Index","2026-02-\u0033\u0030",null]"#)
            .is_err()
    );
}

#[cfg(feature = "news")]
#[test]
fn optional_wire_datetime_preserves_empty_trim_and_parse_errors() {
    for raw in [
        serde_json::Value::Null,
        serde_json::json!(""),
        serde_json::json!("\u{2003} "),
    ] {
        let row: EventRow = serde_json::from_value(serde_json::json!([
            1,
            "site",
            "Event",
            raw,
            "2026-03-06 10:00:00"
        ]))
        .expect("empty optional datetime");
        assert_eq!(row.3, None);
    }
    let row: EventRow =
        serde_json::from_str(r#"[1,"site","Event"," 2026-03-06 10:00:00 ","2026-03-06 10:00:00"]"#)
            .expect("trimmed datetime");
    assert_eq!(row.3, Some(dt("2026-03-06 10:00:00")));
    assert!(
        serde_json::from_str::<EventRow>(
            r#"[1,"site","Event","2026-02-30 10:00:00","2026-03-06 10:00:00"]"#
        )
        .is_err()
    );
}

use super::*;

#[test]
fn single_security_preserves_cardinality_and_endpoint_context() {
    let endpoint = "securities/SBER.json";
    assert_eq!(
        decode_single_security(r#"{"securities":{"data":[]}}"#, endpoint).unwrap(),
        None
    );
    let row = r#"["SBER","Sber","Sberbank","A"]"#;
    let payload = format!(r#"{{"securities":{{"data":[{row}]}}}}"#);
    assert_eq!(
        decode_single_security(&payload, endpoint)
            .unwrap()
            .unwrap()
            .secid()
            .as_str(),
        "SBER"
    );
    let payload = format!(r#"{{"securities":{{"data":[{row},{row}]}}}}"#);
    assert!(matches!(decode_single_security(&payload, endpoint),
        Err(MoexError::UnexpectedSecurityRows { endpoint: actual, row_count: 2 }) if actual.as_ref() == endpoint));
    let payload = format!(r#"{{"securities":{{"data":[{row},["","","",""]]}}}}"#);
    assert!(matches!(
        decode_single_security(&payload, endpoint),
        Err(MoexError::InvalidSecurity { row: 1, .. })
    ));
}

#[cfg(feature = "history")]
#[test]
fn single_history_dates_preserves_cardinality_and_endpoint_context() {
    let endpoint = "history/demo/dates.json";
    assert_eq!(
        decode_single_history_dates(r#"{"dates":{"data":[]}}"#, endpoint).unwrap(),
        None
    );
    let row = r#"["2013-03-25","2026-03-06"]"#;
    let payload = format!(r#"{{"dates":{{"data":[{row}]}}}}"#);
    assert_eq!(
        decode_single_history_dates(&payload, endpoint)
            .unwrap()
            .unwrap()
            .from()
            .to_string(),
        "2013-03-25"
    );
    let payload = format!(r#"{{"dates":{{"data":[{row},{row}]}}}}"#);
    assert!(matches!(decode_single_history_dates(&payload, endpoint),
        Err(MoexError::UnexpectedHistoryDatesRows { endpoint: actual, row_count: 2 }) if actual.as_ref() == endpoint));
    let payload = format!(r#"{{"dates":{{"data":[{row},["2026-03-06","2013-03-25"]]}}}}"#);
    assert!(matches!(
        decode_single_history_dates(&payload, endpoint),
        Err(MoexError::InvalidHistoryDates { row: 1, .. })
    ));
}

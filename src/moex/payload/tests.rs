use super::*;

#[test]
fn snapshots_keep_last_marketdata_row_and_consume_it_once() {
    let payload = r#"{
        "securities": {"data": [["SBER", 10], ["SBER", 1]]},
        "marketdata": {"data": [["SBER", 100], ["SBER", 200], ["GAZP", 300]]}
    }"#;
    let rows = decode_board_security_snapshots_json_with_endpoint(payload, "snapshots.json")
        .expect("valid snapshots");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].secid().as_str(), "SBER");
    assert_eq!(rows[0].last(), Some(200.0));
    assert_eq!(rows[1].secid().as_str(), "SBER");
    assert_eq!(rows[1].last(), None);
    assert_eq!(rows[2].secid().as_str(), "GAZP");
    assert_eq!(rows[2].last(), Some(300.0));
}

#[test]
fn snapshots_report_marketdata_error_before_invalid_security() {
    let payload = r#"{
        "securities": {"data": [["", -1]]},
        "marketdata": {"data": [["SBER", 100], ["", 200]]}
    }"#;
    assert!(matches!(
        decode_board_security_snapshots_json_with_endpoint(payload, "snapshots.json"),
        Err(MoexError::InvalidSecuritySnapshot {
            endpoint,
            table: "marketdata",
            row: 1,
            source: ParseSecuritySnapshotError::InvalidSecId(_),
        }) if endpoint.as_ref() == "snapshots.json"
    ));
}

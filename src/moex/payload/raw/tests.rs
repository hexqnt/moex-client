use serde::Deserialize;
use serde_json::{Value, json};

use crate::moex::{MoexError, decode};

const ENDPOINT: &str = "raw/demo.json";

#[test]
fn selected_table_keeps_first_duplicate_and_skips_unrelated_blocks() {
    let payload = r#"{
        "metadata": [null, {"nested": [false, 1]}],
        "selected": {"columns": ["value"], "data": [[1]]},
        "selected": "ignored duplicate",
        "other": {"columns": false}
    }"#;

    let rows: Vec<Value> = decode::raw_table_rows_json(payload, ENDPOINT, "selected").unwrap();
    assert_eq!(rows, [json!({"value": 1})]);
    let view = decode::raw_table_view_json(payload, ENDPOINT, "selected").unwrap();
    assert_eq!(view.deserialize_value::<u32>(0, "value").unwrap(), Some(1));
}

#[test]
fn selected_table_unescapes_names_and_preserves_borrowed_cells() {
    let payload = r#"{"selec\u0074ed":{"columns":["SEC\u0049D"],"data":[["SBER"]]}}"#;
    let rows: Vec<Value> = decode::raw_table_rows_json(payload, ENDPOINT, "selected").unwrap();
    assert_eq!(rows, [json!({"SECID": "SBER"})]);

    let view = decode::raw_table_view_json(payload, ENDPOINT, "selected").unwrap();
    let secid: &str = view.deserialize_value(0, "SECID").unwrap().unwrap();
    assert_eq!(secid, "SBER");
    let payload_range = payload.as_bytes().as_ptr_range();
    assert!(payload_range.contains(&secid.as_ptr()));
    assert!(view.raw_value(1, "SECID").is_none());
    assert!(view.raw_value(0, "absent").is_none());
}

#[test]
fn selected_table_preserves_trailing_input_behavior() {
    let payload = r#"{"selected":{"columns":[],"data":[]}} trailing input"#;
    assert!(decode::raw_table_rows_json::<Value>(payload, ENDPOINT, "selected").is_ok());
    assert!(decode::raw_table_view_json(payload, ENDPOINT, "selected").is_ok());
    assert!(matches!(
        decode::raw_tables_json(payload, ENDPOINT),
        Err(MoexError::Decode { .. })
    ));
    assert!(matches!(
        decode::engines_json(r#"{"engines":{"data":[]}} trailing input"#),
        Err(MoexError::Decode { .. })
    ));
}

#[test]
fn typed_table_envelope_keeps_missing_and_duplicate_field_diagnostics() {
    for (payload, message) in [
        (r#"{}"#, "missing field `engines`"),
        (
            r#"{"engines":{"data":[]},"engines":{"data":[]}}"#,
            "duplicate field `engines`",
        ),
    ] {
        let MoexError::Decode { endpoint, source } = decode::engines_json(payload).unwrap_err()
        else {
            panic!("expected decode error");
        };
        assert_eq!(endpoint.as_ref(), "engines.json");
        assert!(source.to_string().starts_with(message));
    }
}

#[test]
fn selected_table_preserves_missing_and_decode_error_context() {
    for result in [
        decode::raw_table_rows_json::<Value>("{}", ENDPOINT, "selected").map(|_| ()),
        decode::raw_table_view_json("{}", ENDPOINT, "selected").map(|_| ()),
    ] {
        assert!(matches!(
            result,
            Err(MoexError::MissingRawTable { endpoint, table })
                if endpoint.as_ref() == ENDPOINT && table.as_ref() == "selected"
        ));
    }
    let payload = r#"{"selected":{"columns":true,"data":[]}}"#;
    for result in [
        decode::raw_table_rows_json::<Value>(payload, ENDPOINT, "selected").map(|_| ()),
        decode::raw_table_view_json(payload, ENDPOINT, "selected").map(|_| ()),
    ] {
        assert!(matches!(
            result,
            Err(MoexError::Decode { endpoint, .. }) if endpoint.as_ref() == ENDPOINT
        ));
    }
}

#[test]
fn selected_table_reports_first_short_or_long_row_with_full_context() {
    for (data, width) in [("[[1],[]]", 0), ("[[1],[2,3]]", 2)] {
        let payload = format!(r#"{{"selected":{{"columns":["value"],"data":{data}}}}}"#);
        for result in [
            decode::raw_table_rows_json::<Value>(&payload, ENDPOINT, "selected").map(|_| ()),
            decode::raw_table_view_json(&payload, ENDPOINT, "selected").map(|_| ()),
        ] {
            assert!(matches!(
                result,
                Err(MoexError::InvalidRawTableRowWidth {
                    endpoint, table, row: 1, expected: 1, actual,
                }) if endpoint.as_ref() == ENDPOINT && table.as_ref() == "selected" && actual == width
            ));
        }
    }
}

#[test]
fn owned_table_reports_earlier_value_error_before_later_width_error() {
    #[derive(Debug, Deserialize)]
    struct Row {
        _value: u32,
    }
    let payload = r#"{"selected":{"columns":["_value"],"data":[["invalid"],[]]}}"#;
    assert!(matches!(
        decode::raw_table_rows_json::<Row>(payload, ENDPOINT, "selected"),
        Err(MoexError::InvalidRawTableRow { row: 0, .. })
    ));
}

#[test]
fn selected_table_accepts_empty_columns_and_zero_width_rows() {
    let payload = r#"{"selected":{"columns":[],"data":[[],[]]}}"#;
    let rows: Vec<Value> = decode::raw_table_rows_json(payload, ENDPOINT, "selected").unwrap();
    assert_eq!(rows, [json!({}), json!({})]);
    let view = decode::raw_table_view_json(payload, ENDPOINT, "selected").unwrap();
    assert!(view.columns().is_empty());
    assert_eq!(view.len(), 2);
}

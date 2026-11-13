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
        (
            "true",
            "invalid type: boolean `true`, expected struct EnginesResponse",
        ),
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

#[test]
fn decode_raw_table_rows_maps_columns_to_struct_fields() {
    #[derive(Debug, Deserialize)]
    struct RawHistoryCloseRow {
        #[serde(rename = "SECID")]
        secid: String,
        #[serde(rename = "BOARDID")]
        boardid: String,
        #[serde(rename = "CLOSE")]
        close: Option<f64>,
    }

    let payload = r#"
        {
            "history": {
                "columns": ["SECID", "BOARDID", "CLOSE"],
                "data": [
                    ["SBER", "TQBR", 313.45],
                    ["GAZP", "TQBR", null]
                ]
            }
        }
    "#;

    let rows: Vec<RawHistoryCloseRow> =
        decode::raw_table_rows_json(payload, "history/demo.json", "history")
            .expect("must decode table rows");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].secid, "SBER");
    assert_eq!(rows[0].boardid, "TQBR");
    assert_eq!(rows[0].close, Some(313.45));
    assert_eq!(rows[1].secid, "GAZP");
    assert_eq!(rows[1].close, None);
}

#[test]
fn raw_tables_json_take_rows_decodes_multiple_tables() {
    #[derive(Debug, Deserialize)]
    struct SecurityRow {
        #[serde(rename = "SECID")]
        secid: String,
        #[serde(rename = "LOTSIZE")]
        lot_size: Option<i64>,
    }

    #[derive(Debug, Deserialize)]
    struct MarketDataRow {
        #[serde(rename = "SECID")]
        secid: String,
        #[serde(rename = "LAST")]
        last: Option<f64>,
    }

    let payload = r#"
        {
            "securities": {
                "columns": ["SECID", "LOTSIZE"],
                "data": [
                    ["SBER", 10],
                    ["GAZP", 1]
                ]
            },
            "marketdata": {
                "columns": ["SECID", "LAST"],
                "data": [
                    ["SBER", 313.45],
                    ["GAZP", null]
                ]
            }
        }
    "#;

    let mut tables = decode::raw_tables_json(payload, "board/snapshots.json")
        .expect("must decode top-level raw tables");
    assert_eq!(tables.len(), 2);
    assert!(!tables.is_empty());

    let security_rows: Vec<SecurityRow> = tables
        .take_rows("securities")
        .expect("must decode securities rows");
    let marketdata_rows: Vec<MarketDataRow> = tables
        .take_rows("marketdata")
        .expect("must decode marketdata rows");

    assert_eq!(security_rows.len(), 2);
    assert_eq!(security_rows[0].secid, "SBER");
    assert_eq!(security_rows[0].lot_size, Some(10));
    assert_eq!(marketdata_rows.len(), 2);
    assert_eq!(marketdata_rows[0].secid, "SBER");
    assert_eq!(marketdata_rows[0].last, Some(313.45));
    assert_eq!(marketdata_rows[1].last, None);
    assert!(tables.is_empty());
}

#[test]
fn raw_tables_json_take_rows_reports_missing_table() {
    let payload = r#"
        {
            "history": {
                "columns": ["SECID"],
                "data": [["SBER"]]
            }
        }
    "#;

    let mut tables =
        decode::raw_tables_json(payload, "history/demo.json").expect("must decode top-level map");
    let err = tables
        .take_rows::<serde_json::Value>("securities")
        .expect_err("missing table must fail");

    assert!(matches!(
        err,
        MoexError::MissingRawTable {
            table,
            endpoint: _
        } if table.as_ref() == "securities"
    ));
}

#[test]
fn raw_tables_json_take_rows_reports_decode_error_for_non_table_block() {
    let payload = r#"
        {
            "meta": {
                "foo": "bar"
            }
        }
    "#;

    let mut tables =
        decode::raw_tables_json(payload, "meta/demo.json").expect("must decode top-level map");
    let err = tables
        .take_rows::<serde_json::Value>("meta")
        .expect_err("non-table block must fail");

    assert!(matches!(
        err,
        MoexError::Decode { endpoint, .. } if endpoint.as_ref() == "meta/demo.json (table=meta)"
    ));
}

#[test]
fn decode_raw_table_view_provides_borrowed_column_access() {
    let payload = r#"
        {
            "history": {
                "columns": ["SECID", "BOARDID", "CLOSE"],
                "data": [
                    ["SBER", "TQBR", 313.45],
                    ["GAZP", "TQBR", null]
                ]
            }
        }
    "#;

    let table = decode::raw_table_view_json(payload, "history/demo.json", "history")
        .expect("must decode borrowed table view");
    assert_eq!(table.len(), 2);
    assert_eq!(table.columns().len(), 3);
    assert_eq!(table.columns()[0].as_ref(), "SECID");

    let secid: &str = table
        .deserialize_value(0, "SECID")
        .expect("must decode secid")
        .expect("must contain secid");
    assert_eq!(secid, "SBER");

    let close: Option<Option<f64>> = table
        .deserialize_value(1, "CLOSE")
        .expect("must decode close column");
    assert_eq!(close, Some(None));
}

#[test]
fn decode_raw_table_rows_reports_missing_table() {
    let payload = r#"
        {
            "history": {
                "columns": ["SECID"],
                "data": [["SBER"]]
            }
        }
    "#;

    let err = decode::raw_table_rows_json::<serde_json::Value>(
        payload,
        "history/demo.json",
        "securities",
    )
    .expect_err("missing table must fail");

    assert!(matches!(
        err,
        MoexError::MissingRawTable {
            ref table,
            endpoint: _
        } if table.as_ref() == "securities"
    ));
}

#[test]
fn decode_raw_table_rows_reports_width_mismatch() {
    let payload = r#"
        {
            "history": {
                "columns": ["SECID", "BOARDID", "CLOSE"],
                "data": [["SBER", "TQBR"]]
            }
        }
    "#;

    let err =
        decode::raw_table_rows_json::<serde_json::Value>(payload, "history/demo.json", "history")
            .expect_err("row width mismatch must fail");

    assert!(matches!(
        err,
        MoexError::InvalidRawTableRowWidth {
            row,
            expected,
            actual,
            endpoint: _,
            table: _
        } if row == 0 && expected == 3 && actual == 2
    ));
}

#[test]
fn decode_raw_table_rows_reports_row_decode_error() {
    #[derive(Debug, Deserialize)]
    struct StrictRow {
        #[serde(rename = "SECID")]
        _secid: String,
        #[serde(rename = "CLOSE")]
        _close: f64,
    }

    let payload = r#"
        {
            "history": {
                "columns": ["SECID", "CLOSE"],
                "data": [["SBER", "not-a-number"]]
            }
        }
    "#;

    let err = decode::raw_table_rows_json::<StrictRow>(payload, "history/demo.json", "history")
        .expect_err("invalid row value must fail");

    assert!(matches!(
        err,
        MoexError::InvalidRawTableRow {
            row,
            endpoint: _,
            table: _,
            source: _
        } if row == 0
    ));
}

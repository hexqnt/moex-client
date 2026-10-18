use serde::Deserialize;

use crate::models::{BoardId, EngineName, IndexId, MarketName, SecId};
use crate::moex::{IssEndpoint, decode};

use super::*;

#[test]
fn market_securities_endpoint_uses_engine_and_market() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");

    let endpoint = market_securities_endpoint(&engine, &market);
    assert_eq!(endpoint, "engines/stock/markets/shares/securities.json");
}

#[test]
fn market_security_endpoint_uses_engine_market_and_security() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let secid = SecId::try_from("SBER").expect("valid secid");

    let endpoint = market_security_endpoint(&engine, &market, &secid);
    assert_eq!(
        endpoint,
        "engines/stock/markets/shares/securities/SBER.json"
    );
}

#[test]
fn market_orderbook_endpoint_uses_engine_and_market() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");

    let endpoint = market_orderbook_endpoint(&engine, &market);
    assert_eq!(endpoint, "engines/stock/markets/shares/orderbook.json");
}

#[test]
fn market_trades_endpoint_uses_engine_and_market() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");

    let endpoint = market_trades_endpoint(&engine, &market);
    assert_eq!(endpoint, "engines/stock/markets/shares/trades.json");
}

#[test]
fn candleborders_endpoint_uses_engine_market_and_security() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let secid = SecId::try_from("SBER").expect("valid secid");

    let endpoint = candleborders_endpoint(&engine, &market, &secid);
    assert_eq!(
        endpoint,
        "engines/stock/markets/shares/securities/SBER/candleborders.json"
    );
}

#[cfg(feature = "history")]
#[test]
fn history_endpoint_uses_engine_market_board_and_security() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let secid = SecId::try_from("SBER").expect("valid secid");

    let endpoint = history_endpoint(&engine, &market, &board, &secid);
    assert_eq!(
        endpoint,
        "history/engines/stock/markets/shares/boards/TQBR/securities/SBER.json"
    );
}

#[test]
fn engine_turnovers_endpoint_uses_engine() {
    let engine = EngineName::try_from("stock").expect("valid engine");

    let endpoint = engine_turnovers_endpoint(&engine);
    assert_eq!(endpoint, "engines/stock/turnovers.json");
}

#[test]
fn secstats_endpoint_uses_engine_and_market() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");

    let endpoint = secstats_endpoint(&engine, &market);
    assert_eq!(endpoint, "engines/stock/markets/shares/secstats.json");
}

#[cfg(feature = "news")]
#[test]
fn sitenews_endpoint_is_stable() {
    assert_eq!(SITENEWS_ENDPOINT, "sitenews.json");
}

#[cfg(feature = "news")]
#[test]
fn events_endpoint_is_stable() {
    assert_eq!(EVENTS_ENDPOINT, "events.json");
}

#[test]
fn global_securities_endpoint_is_stable() {
    assert_eq!(GLOBAL_SECURITIES_ENDPOINT, "securities.json");
}

#[test]
fn iss_endpoint_maps_to_expected_path_and_default_table() {
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let secid = SecId::try_from("SBER").expect("valid secid");
    let index = IndexId::try_from("IMOEX").expect("valid index");

    let endpoint = IssEndpoint::Securities {
        engine: &engine,
        market: &market,
        board: &board,
    };
    assert_eq!(
        endpoint.path(),
        "engines/stock/markets/shares/boards/TQBR/securities.json"
    );
    assert_eq!(endpoint.default_table(), Some("securities"));

    let snapshots = IssEndpoint::BoardSecuritySnapshots {
        engine: &engine,
        market: &market,
        board: &board,
    };
    assert_eq!(
        snapshots.path(),
        "engines/stock/markets/shares/boards/TQBR/securities.json"
    );
    assert_eq!(snapshots.default_table(), Some("securities,marketdata"));

    let analytics = IssEndpoint::IndexAnalytics { indexid: &index };
    assert_eq!(
        analytics.path(),
        "statistics/engines/stock/markets/index/analytics/IMOEX.json"
    );
    assert_eq!(analytics.default_table(), Some("analytics"));

    let security_info = IssEndpoint::SecurityInfo { security: &secid };
    assert_eq!(security_info.path(), "securities/SBER.json");
    assert_eq!(security_info.default_table(), Some("securities"));
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
        decode_raw_table_rows_json_with_endpoint(payload, "history/demo.json", "history")
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

    let err = decode_raw_table_rows_json_with_endpoint::<serde_json::Value>(
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

    let err = decode_raw_table_rows_json_with_endpoint::<serde_json::Value>(
        payload,
        "history/demo.json",
        "history",
    )
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

    let err = decode_raw_table_rows_json_with_endpoint::<StrictRow>(
        payload,
        "history/demo.json",
        "history",
    )
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

#![cfg(feature = "blocking")]

use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use chrono::{NaiveDate, NaiveDateTime};
use reqwest::{
    StatusCode, Url,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::Deserialize;

#[cfg(feature = "history")]
use crate::models::HistoryDates;
use crate::models::{
    BoardId, CandleInterval, CandleQuery, EngineName, IndexId, MarketName, Pagination, SecId,
    Security,
};

#[cfg(feature = "history")]
use super::client::optional_single_history_dates;
use super::client::{
    append_candle_query_to_url, append_pagination_to_url, looks_like_json_payload,
    normalize_raw_endpoint_path, optional_single_security, truncate_prefix,
};
use super::constants::*;
use super::convert::*;
use super::decode;
use super::payload::decode_raw_table_rows_json_with_endpoint;
use super::wire::*;
use super::*;

fn d(input: &str) -> NaiveDate {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").unwrap()
}

fn dt(input: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap()
}

#[test]
fn parse_typical_iss_indices_payload() {
    let payload = r#"
        {
            "indices": {
                "columns": ["indexid", "shortname", "from", "till"],
                "data": [
                    ["IMOEX", "Индекс МосБиржи", "2001-01-03", "2026-03-05"],
                    ["RTSI", "Индекс РТС", "1995-09-01", ""]
                ]
            }
        }
        "#;
    let response: IndexesResponse = serde_json::from_str(payload).expect("valid payload");
    let indexes =
        convert_index_rows(response.indices.data, INDEXES_ENDPOINT).expect("valid indexes");

    assert_eq!(indexes.len(), 2);
    assert_eq!(indexes[0].id().as_str(), "IMOEX");
    assert_eq!(indexes[0].from(), Some(d("2001-01-03")));
    assert_eq!(indexes[0].till(), Some(d("2026-03-05")));
    assert_eq!(indexes[1].till(), None);
}

#[cfg(feature = "history")]
#[test]
fn parse_typical_iss_history_dates_payload() {
    let payload = r#"
        {
            "dates": {
                "columns": ["from", "till"],
                "data": [
                    ["2013-03-25", "2026-03-06"]
                ]
            }
        }
        "#;
    let dates = decode::history_dates_json(payload).expect("valid payload");
    assert_eq!(dates.len(), 1);
    assert_eq!(dates[0].from(), d("2013-03-25"));
    assert_eq!(dates[0].till(), d("2026-03-06"));
}

#[cfg(feature = "history")]
#[test]
fn invalid_history_dates_row_reports_row_number() {
    let payload = r#"
        {
            "dates": {
                "columns": ["from", "till"],
                "data": [
                    ["2026-03-07", "2026-03-06"]
                ]
            }
        }
        "#;
    let err = decode::history_dates_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidHistoryDates { row, .. } => assert_eq!(row, 0),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[cfg(feature = "history")]
#[test]
fn parse_typical_iss_history_payload() {
    let payload = r#"
        {
            "history": {
                "columns": ["BOARDID", "TRADEDATE", "SECID", "NUMTRADES", "VALUE", "OPEN", "LOW", "HIGH", "CLOSE", "VOLUME"],
                "data": [
                    ["TQBR", "2026-03-05", "SBER", 120345, 123456789.5, 314.0, 310.0, 315.2, 314.8, 3900000],
                    ["TQBR", "2026-03-06", "SBER", 118000, 118000000.0, 314.8, 311.5, 316.0, 315.3, 3700000]
                ]
            }
        }
        "#;
    let history = decode::history_json(payload).expect("valid payload");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].boardid().as_str(), "TQBR");
    assert_eq!(history[0].secid().as_str(), "SBER");
    assert_eq!(history[0].tradedate(), d("2026-03-05"));
    assert_eq!(history[1].numtrades(), Some(118_000));
}

#[cfg(feature = "history")]
#[test]
fn invalid_history_row_reports_row_number() {
    let payload = r#"
        {
            "history": {
                "columns": ["BOARDID", "TRADEDATE", "SECID", "NUMTRADES", "VALUE", "OPEN", "LOW", "HIGH", "CLOSE", "VOLUME"],
                "data": [
                    ["TQBR", "2026-03-05", "SBER", -1, 123456789.5, 314.0, 310.0, 315.2, 314.8, 3900000]
                ]
            }
        }
        "#;
    let err = decode::history_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidHistory { row, .. } => assert_eq!(row, 0),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_turnovers_payload() {
    let payload = r#"
        {
            "turnovers": {
                "columns": ["NAME", "ID", "VALTODAY", "VALTODAY_USD", "NUMTRADES", "UPDATETIME", "TITLE"],
                "data": [
                    ["stock", 1, null, null, null, "2026-03-07 07:00:00", "Фондовый рынок и рынок депозитов"],
                    ["currency", 3, 123456789.5, 1500000.0, 32000, "2026-03-06 23:50:23", "Валютный рынок"]
                ]
            }
        }
        "#;
    let turnovers = decode::turnovers_json(payload).expect("valid payload");
    assert_eq!(turnovers.len(), 2);
    assert_eq!(turnovers[0].name(), "stock");
    assert_eq!(turnovers[1].id(), 3);
    assert_eq!(turnovers[1].numtrades(), Some(32_000));
}

#[test]
fn invalid_turnover_row_reports_row_number() {
    let payload = r#"
        {
            "turnovers": {
                "columns": ["NAME", "ID", "VALTODAY", "VALTODAY_USD", "NUMTRADES", "UPDATETIME", "TITLE"],
                "data": [
                    ["currency", 3, 123456789.5, 1500000.0, -1, "2026-03-06 23:50:23", "Валютный рынок"]
                ]
            }
        }
        "#;
    let err = decode::turnovers_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidTurnover { row, .. } => assert_eq!(row, 0),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[cfg(feature = "news")]
#[test]
fn parse_typical_iss_sitenews_payload() {
    let payload = r#"
        {
            "sitenews": {
                "columns": ["id", "tag", "title", "published_at", "modified_at"],
                "data": [
                    [98236, "site", "О дополнительных условиях проведения торгов", "2026-03-06 19:08:57", "2026-03-06 19:08:57"],
                    [98235, "site", "Техническое объявление", "2026-03-06 18:00:00", "2026-03-06 18:00:00"]
                ]
            }
        }
        "#;
    let news = decode::sitenews_json(payload).expect("valid payload");
    assert_eq!(news.len(), 2);
    assert_eq!(news[0].id(), 98_236);
    assert_eq!(news[1].tag(), "site");
}

#[cfg(feature = "news")]
#[test]
fn invalid_sitenews_row_reports_row_number() {
    let payload = r#"
        {
            "sitenews": {
                "columns": ["id", "tag", "title", "published_at", "modified_at"],
                "data": [
                    [98236, "site", "  ", "2026-03-06 19:08:57", "2026-03-06 19:08:57"]
                ]
            }
        }
        "#;
    let err = decode::sitenews_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidSiteNews { row, .. } => assert_eq!(row, 0),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[cfg(feature = "news")]
#[test]
fn parse_typical_iss_events_payload() {
    let payload = r#"
        {
            "events": {
                "columns": ["id", "tag", "title", "from", "modified_at"],
                "data": [
                    [77, "site", "Технические работы", "2026-03-07 10:00:00", "2026-03-06 22:00:00"],
                    [78, "site", "Событие без from", null, "2026-03-06 21:00:00"]
                ]
            }
        }
        "#;
    let events = decode::events_json(payload).expect("valid payload");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].id(), 77);
    assert_eq!(events[1].from(), None);
}

#[cfg(feature = "news")]
#[test]
fn invalid_event_row_reports_row_number() {
    let payload = r#"
        {
            "events": {
                "columns": ["id", "tag", "title", "from", "modified_at"],
                "data": [
                    [77, " ", "Технические работы", null, "2026-03-06 22:00:00"]
                ]
            }
        }
        "#;
    let err = decode::events_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidEvent { row, .. } => assert_eq!(row, 0),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_secstats_payload() {
    let payload = r#"
        {
            "secstats": {
                "columns": ["SECID", "BOARDID", "VOLTODAY", "VALTODAY", "HIGHBID", "LOWOFFER", "LASTOFFER", "LASTBID", "OPEN", "LOW", "HIGH", "LAST", "NUMTRADES", "WAPRICE"],
                "data": [
                    ["SBER", "TQBR", 12500000, 3950000000.5, 314.79, 314.8, 314.8, 314.79, 313.0, 312.4, 315.0, 314.8, 157809, 314.55],
                    ["GAZP", "TQBR", 8200000, 1450000000.0, 171.0, 171.2, 171.2, 171.0, 170.4, 169.8, 171.5, 171.1, 100500, 171.02]
                ]
            }
        }
        "#;
    let stats = decode::secstats_json(payload).expect("valid payload");
    assert_eq!(stats.len(), 2);
    assert_eq!(stats[0].secid().as_str(), "SBER");
    assert_eq!(stats[0].boardid().as_str(), "TQBR");
    assert_eq!(stats[1].numtrades(), Some(100_500));
}

#[test]
fn invalid_secstats_row_reports_row_number() {
    let payload = r#"
        {
            "secstats": {
                "columns": ["SECID", "BOARDID", "VOLTODAY", "VALTODAY", "HIGHBID", "LOWOFFER", "LASTOFFER", "LASTBID", "OPEN", "LOW", "HIGH", "LAST", "NUMTRADES", "WAPRICE"],
                "data": [
                    ["SBER", "TQBR", -1, 3950000000.5, 314.79, 314.8, 314.8, 314.79, 313.0, 312.4, 315.0, 314.8, 157809, 314.55]
                ]
            }
        }
        "#;
    let err = decode::secstats_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidSecStat { row, .. } => assert_eq!(row, 0),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn invalid_row_reports_row_number() {
    let payload = r#"
        {
            "indices": {
                "columns": ["indexid", "shortname", "from", "till"],
                "data": [
                    ["IMOEX", "Индекс МосБиржи", "2001-01-03", "2026-03-05"],
                    ["", "Broken", "2001-01-03", "2026-03-05"]
                ]
            }
        }
        "#;
    let response: IndexesResponse = serde_json::from_str(payload).expect("valid payload");
    let err = convert_index_rows(response.indices.data, INDEXES_ENDPOINT).expect_err("invalid row");

    match err {
        MoexError::InvalidIndex { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_index_analytics_payload() {
    let payload = r#"
        {
            "analytics": {
                "columns": ["indexid", "tradedate", "ticker", "shortnames", "secids", "weight", "tradingsession", "trade_session_date"],
                "data": [
                    ["IMOEX", "2026-03-06", "SBER", "Сбербанк", "SBER", 14.56, 3, "2026-03-06"],
                    ["IMOEX", "2026-03-06", "GAZP", "ГАЗПРОМ ао", "GAZP", 9.80, 3, "2026-03-06"]
                ]
            }
        }
        "#;

    let response: IndexAnalyticsResponse = serde_json::from_str(payload).expect("valid payload");
    let analytics =
        convert_index_analytics_rows(response.analytics.data, INDEX_ANALYTICS_ENDPOINT_TEMPLATE)
            .expect("valid analytics rows");

    assert_eq!(analytics.len(), 2);
    assert_eq!(analytics[0].indexid().as_str(), "IMOEX");
    assert_eq!(analytics[0].ticker().as_str(), "SBER");
    assert_eq!(analytics[0].weight(), 14.56);
    assert_eq!(analytics[1].secid().as_str(), "GAZP");
    assert_eq!(analytics[1].trade_session_date(), d("2026-03-06"));
}

#[test]
fn invalid_index_analytics_row_reports_row_number() {
    let payload = r#"
        {
            "analytics": {
                "columns": ["indexid", "tradedate", "ticker", "shortnames", "secids", "weight", "tradingsession", "trade_session_date"],
                "data": [
                    ["IMOEX", "2026-03-06", "SBER", "Сбербанк", "SBER", 14.56, 3, "2026-03-06"],
                    ["IMOEX", "2026-03-06", "GAZP", "ГАЗПРОМ ао", "GAZP", -9.8, 3, "2026-03-06"]
                ]
            }
        }
        "#;

    let response: IndexAnalyticsResponse = serde_json::from_str(payload).expect("valid payload");
    let err =
        convert_index_analytics_rows(response.analytics.data, INDEX_ANALYTICS_ENDPOINT_TEMPLATE)
            .expect_err("invalid second analytics row");

    match err {
        MoexError::InvalidIndexAnalytics { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_engines_payload() {
    let payload = r#"
        {
            "engines": {
                "columns": ["id", "name", "title"],
                "data": [
                    [1, "stock", "Фондовый рынок и рынок депозитов"],
                    [4, "futures", "Срочный рынок"]
                ]
            }
        }
        "#;
    let response: EnginesResponse = serde_json::from_str(payload).expect("valid payload");
    let engines =
        convert_engine_rows(response.engines.data, ENGINES_ENDPOINT).expect("valid engines");

    assert_eq!(engines.len(), 2);
    assert_eq!(engines[0].id().get(), 1);
    assert_eq!(engines[0].name().as_str(), "stock");
}

#[test]
fn invalid_engine_row_reports_row_number() {
    let payload = r#"
        {
            "engines": {
                "columns": ["id", "name", "title"],
                "data": [
                    [1, "stock", "Фондовый рынок и рынок депозитов"],
                    [0, "broken", "Broken"]
                ]
            }
        }
        "#;
    let response: EnginesResponse = serde_json::from_str(payload).expect("valid payload");
    let err =
        convert_engine_rows(response.engines.data, ENGINES_ENDPOINT).expect_err("invalid row");

    match err {
        MoexError::InvalidEngine { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_markets_payload() {
    let payload = r#"
        {
            "markets": {
                "columns": ["id", "NAME", "title"],
                "data": [
                    [5, "index", "Индексы фондового рынка"],
                    [1, "shares", "Рынок акций"]
                ]
            }
        }
        "#;
    let response: MarketsResponse = serde_json::from_str(payload).expect("valid payload");
    let markets = convert_market_rows(response.markets.data, MARKETS_ENDPOINT_TEMPLATE)
        .expect("valid markets");

    assert_eq!(markets.len(), 2);
    assert_eq!(markets[0].id().get(), 5);
    assert_eq!(markets[0].name().as_str(), "index");
}

#[test]
fn invalid_market_row_reports_row_number() {
    let payload = r#"
        {
            "markets": {
                "columns": ["id", "NAME", "title"],
                "data": [
                    [5, "index", "Индексы фондового рынка"],
                    [0, "broken", "Broken"]
                ]
            }
        }
        "#;
    let response: MarketsResponse = serde_json::from_str(payload).expect("valid payload");
    let err = convert_market_rows(response.markets.data, MARKETS_ENDPOINT_TEMPLATE)
        .expect_err("invalid row");

    match err {
        MoexError::InvalidMarket { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_boards_payload() {
    let payload = r#"
        {
            "boards": {
                "columns": ["id", "board_group_id", "boardid", "title", "is_traded"],
                "data": [
                    [95, 30, "EQCC", "ЦК - режим основных торгов - безадрес.", 0],
                    [7, 13, "EQRP", "РПС - Акции и паи", 1]
                ]
            }
        }
        "#;
    let response: BoardsResponse = serde_json::from_str(payload).expect("valid payload");
    let boards =
        convert_board_rows(response.boards.data, BOARDS_ENDPOINT_TEMPLATE).expect("valid boards");

    assert_eq!(boards.len(), 2);
    assert_eq!(boards[0].id(), 95);
    assert_eq!(boards[0].boardid().as_str(), "EQCC");
    assert!(!boards[0].is_traded());
}

#[test]
fn invalid_board_row_reports_row_number() {
    let payload = r#"
        {
            "boards": {
                "columns": ["id", "board_group_id", "boardid", "title", "is_traded"],
                "data": [
                    [95, 30, "EQCC", "ЦК - режим основных торгов - безадрес.", 0],
                    [7, 13, "EQRP", "РПС - Акции и паи", 2]
                ]
            }
        }
        "#;
    let response: BoardsResponse = serde_json::from_str(payload).expect("valid payload");
    let err = convert_board_rows(response.boards.data, BOARDS_ENDPOINT_TEMPLATE)
        .expect_err("invalid row");

    match err {
        MoexError::InvalidBoard { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_security_boards_payload() {
    let payload = r#"
        {
            "boards": {
                "columns": ["engine", "market", "boardid", "is_primary"],
                "data": [
                    ["stock", "shares", "TQBR", 1],
                    ["currency", "selt", "CETS", 0]
                ]
            }
        }
        "#;

    let boards = decode::security_boards_json(payload).expect("valid payload");
    assert_eq!(boards.len(), 2);
    assert_eq!(boards[0].engine().as_str(), "stock");
    assert_eq!(boards[0].market().as_str(), "shares");
    assert_eq!(boards[0].boardid().as_str(), "TQBR");
    assert!(boards[0].is_primary());
}

#[test]
fn invalid_security_board_row_reports_row_number() {
    let payload = r#"
        {
            "boards": {
                "columns": ["engine", "market", "boardid", "is_primary"],
                "data": [
                    ["stock", "shares", "TQBR", 1],
                    ["stock", "shares", "TQTF", 2]
                ]
            }
        }
        "#;

    let err = decode::security_boards_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidSecurityBoard { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_securities_payload() {
    let payload = r#"
        {
            "securities": {
                "columns": ["SECID", "SHORTNAME", "SECNAME", "STATUS"],
                "data": [
                    ["ABIO", "iАРТГЕН ао", "ПАО \"Артген\"", "A"],
                    ["AMEZ", "АшинскийМЗ", "Ашинский метзавод ПАО ао", "N"]
                ]
            }
        }
        "#;
    let response: SecuritiesResponse = serde_json::from_str(payload).expect("valid payload");
    let securities = convert_security_rows(response.securities.data, SECURITIES_ENDPOINT_TEMPLATE)
        .expect("valid securities");

    assert_eq!(securities.len(), 2);
    assert_eq!(securities[0].secid().as_str(), "ABIO");
    assert_eq!(securities[1].status(), "N");
}

#[test]
fn invalid_security_row_reports_row_number() {
    let payload = r#"
        {
            "securities": {
                "columns": ["SECID", "SHORTNAME", "SECNAME", "STATUS"],
                "data": [
                    ["ABIO", "iАРТГЕН ао", "ПАО \"Артген\"", "A"],
                    ["AMEZ", "АшинскийМЗ", "Ашинский метзавод ПАО ао", ""]
                ]
            }
        }
        "#;
    let response: SecuritiesResponse = serde_json::from_str(payload).expect("valid payload");
    let err = convert_security_rows(response.securities.data, SECURITIES_ENDPOINT_TEMPLATE)
        .expect_err("invalid row");

    match err {
        MoexError::InvalidSecurity { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_board_security_snapshots_payload() {
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
                    ["SBER", 314.8],
                    ["LKOH", 7000.0]
                ]
            }
        }
        "#;

    let snapshots = decode::board_security_snapshots_json(payload).expect("valid payload");
    assert_eq!(snapshots.len(), 3);

    let sber = snapshots
        .iter()
        .find(|item| item.secid().as_str() == "SBER")
        .expect("SBER must be present");
    assert_eq!(sber.lot_size(), Some(10));
    assert_eq!(sber.last(), Some(314.8));

    let gazp = snapshots
        .iter()
        .find(|item| item.secid().as_str() == "GAZP")
        .expect("GAZP must be present");
    assert_eq!(gazp.lot_size(), Some(1));
    assert_eq!(gazp.last(), None);

    let lkoh = snapshots
        .iter()
        .find(|item| item.secid().as_str() == "LKOH")
        .expect("LKOH must be present");
    assert_eq!(lkoh.lot_size(), None);
    assert_eq!(lkoh.last(), Some(7000.0));
}

#[test]
fn invalid_board_security_snapshot_row_reports_table_and_row() {
    let payload = r#"
        {
            "securities": {
                "columns": ["SECID", "LOTSIZE"],
                "data": [
                    ["SBER", -1]
                ]
            },
            "marketdata": {
                "columns": ["SECID", "LAST"],
                "data": []
            }
        }
        "#;

    let err = decode::board_security_snapshots_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidSecuritySnapshot { table, row, .. } => {
            assert_eq!(table, "securities");
            assert_eq!(row, 0);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_orderbook_payload() {
    let payload = r#"
        {
            "orderbook": {
                "columns": ["BUYSELL", "PRICE", "QUANTITY"],
                "data": [
                    ["B", 314.79, 1500],
                    ["S", 314.80, 2000]
                ]
            }
        }
        "#;
    let response: OrderbookResponse = serde_json::from_str(payload).expect("valid payload");
    let levels = convert_orderbook_rows(response.orderbook.data, ORDERBOOK_ENDPOINT_TEMPLATE)
        .expect("valid orderbook levels");

    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0].buy_sell().as_str(), "B");
    assert_eq!(levels[0].price(), 314.79);
    assert_eq!(levels[0].quantity(), 1500);
}

#[test]
fn parse_typical_iss_candle_borders_payload() {
    let payload = r#"
        {
            "borders": {
                "columns": ["begin", "end", "interval", "board_group_id"],
                "data": [
                    ["2011-12-15 10:00:00", "2026-03-06 23:49:00", 1, 57],
                    ["2007-07-20 00:00:00", "2026-03-06 00:00:00", 24, 57]
                ]
            }
        }
        "#;
    let borders = decode::candle_borders_json(payload).expect("valid payload");
    assert_eq!(borders.len(), 2);
    assert_eq!(borders[0].interval(), CandleInterval::Minute1);
    assert_eq!(borders[0].board_group_id(), 57);
    assert_eq!(borders[1].interval(), CandleInterval::Day1);
}

#[test]
fn invalid_candle_border_row_reports_row_number() {
    let payload = r#"
        {
            "borders": {
                "columns": ["begin", "end", "interval", "board_group_id"],
                "data": [
                    ["2011-12-15 10:00:00", "2026-03-06 23:49:00", 999, 57]
                ]
            }
        }
        "#;
    let err = decode::candle_borders_json(payload).expect_err("invalid row");
    match err {
        MoexError::InvalidCandleBorder { row, .. } => assert_eq!(row, 0),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn invalid_orderbook_row_reports_row_number() {
    let payload = r#"
        {
            "orderbook": {
                "columns": ["BUYSELL", "PRICE", "QUANTITY"],
                "data": [
                    ["B", 314.79, 1500],
                    ["X", 314.80, 2000]
                ]
            }
        }
        "#;
    let response: OrderbookResponse = serde_json::from_str(payload).expect("valid payload");
    let err = convert_orderbook_rows(response.orderbook.data, ORDERBOOK_ENDPOINT_TEMPLATE)
        .expect_err("invalid row");

    match err {
        MoexError::InvalidOrderbook { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_candles_payload() {
    let payload = r#"
        {
            "candles": {
                "columns": ["begin", "end", "open", "close", "high", "low", "value", "volume"],
                "data": [
                    ["2007-07-20 00:00:00", "2007-07-20 23:59:59", 109, 109.2, 111.11, 108.01, 2211623351.09, 20252489],
                    ["2007-07-23 00:00:00", "2007-07-23 23:59:59", 109.7, 112, 112.65, 108.1, 3901828829.37, 35092029]
                ]
            }
        }
        "#;
    let response: CandlesResponse = serde_json::from_str(payload).expect("valid payload");
    let candles = convert_candle_rows(response.candles.data, CANDLES_ENDPOINT_TEMPLATE)
        .expect("valid candles");

    assert_eq!(candles.len(), 2);
    assert_eq!(candles[0].close(), Some(109.2));
    assert_eq!(candles[1].volume(), Some(35_092_029));
}

#[test]
fn invalid_candle_row_reports_row_number() {
    let payload = r#"
        {
            "candles": {
                "columns": ["begin", "end", "open", "close", "high", "low", "value", "volume"],
                "data": [
                    ["2007-07-20 00:00:00", "2007-07-20 23:59:59", 109, 109.2, 111.11, 108.01, 2211623351.09, 20252489],
                    ["2007-07-23 00:00:00", "2007-07-23 23:59:59", 109.7, 112, 112.65, 108.1, 3901828829.37, -1]
                ]
            }
        }
        "#;
    let response: CandlesResponse = serde_json::from_str(payload).expect("valid payload");
    let err = convert_candle_rows(response.candles.data, CANDLES_ENDPOINT_TEMPLATE)
        .expect_err("invalid row");

    match err {
        MoexError::InvalidCandle { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn parse_typical_iss_trades_payload() {
    let payload = r#"
        {
            "trades": {
                "columns": ["TRADENO", "TRADETIME", "PRICE", "QUANTITY", "VALUE"],
                "data": [
                    [15780983820, "06:59:49", 314.8, 1, 314.8],
                    [15780983825, "06:59:49", 314.8, 1, 314.8]
                ]
            }
        }
        "#;
    let response: TradesResponse = serde_json::from_str(payload).expect("valid payload");
    let trades =
        convert_trade_rows(response.trades.data, TRADES_ENDPOINT_TEMPLATE).expect("valid trades");

    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].tradeno(), 15_780_983_820);
    assert_eq!(trades[0].price(), Some(314.8));
}

#[test]
fn invalid_trade_row_reports_row_number() {
    let payload = r#"
        {
            "trades": {
                "columns": ["TRADENO", "TRADETIME", "PRICE", "QUANTITY", "VALUE"],
                "data": [
                    [15780983820, "06:59:49", 314.8, 1, 314.8],
                    [15780983825, "06:59:49", 314.8, -1, 314.8]
                ]
            }
        }
        "#;
    let response: TradesResponse = serde_json::from_str(payload).expect("valid payload");
    let err = convert_trade_rows(response.trades.data, TRADES_ENDPOINT_TEMPLATE)
        .expect_err("invalid row");

    match err {
        MoexError::InvalidTrade { row, .. } => assert_eq!(row, 1),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

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
fn raw_endpoint_builder_accepts_typed_endpoint() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");
    let engine = EngineName::try_from("stock").expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let board = BoardId::try_from("TQBR").expect("valid board");

    let _request = client
        .raw_endpoint(IssEndpoint::Securities {
            engine: &engine,
            market: &market,
            board: &board,
        })
        .columns("securities", "SECID,LOTSIZE");
}

#[test]
fn optional_single_security_accepts_empty_and_single_row() {
    let empty = optional_single_security("securities/SBER.json", Vec::new())
        .expect("empty securities table is valid");
    assert!(empty.is_none());

    let single_row = vec![
        Security::try_new(
            "SBER".to_owned(),
            "Сбербанк".to_owned(),
            "ПАО Сбербанк".to_owned(),
            "A".to_owned(),
        )
        .expect("valid security"),
    ];
    let single = optional_single_security("securities/SBER.json", single_row)
        .expect("single row must be accepted");
    assert_eq!(
        single
            .expect("single row must produce security")
            .secid()
            .as_str(),
        "SBER"
    );
}

#[test]
fn optional_single_security_rejects_multiple_rows() {
    let rows = vec![
        Security::try_new(
            "SBER".to_owned(),
            "Сбербанк".to_owned(),
            "ПАО Сбербанк".to_owned(),
            "A".to_owned(),
        )
        .expect("valid security"),
        Security::try_new(
            "GAZP".to_owned(),
            "Газпром".to_owned(),
            "ПАО Газпром".to_owned(),
            "A".to_owned(),
        )
        .expect("valid security"),
    ];

    let err = optional_single_security("securities/SBER.json", rows)
        .expect_err("multiple rows must be rejected");
    match err {
        MoexError::UnexpectedSecurityRows {
            endpoint,
            row_count,
        } => {
            assert_eq!(endpoint.as_ref(), "securities/SBER.json");
            assert_eq!(row_count, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[cfg(feature = "history")]
#[test]
fn optional_single_history_dates_accepts_empty_and_single_row() {
    let empty = optional_single_history_dates("history/.../dates.json", Vec::new())
        .expect("empty dates table is valid");
    assert!(empty.is_none());

    let single_row =
        vec![HistoryDates::try_new(d("2013-03-25"), d("2026-03-06")).expect("valid dates")];
    let single = optional_single_history_dates("history/.../dates.json", single_row)
        .expect("single row must be accepted");
    assert_eq!(
        single.expect("single row must produce dates").from(),
        d("2013-03-25")
    );
}

#[cfg(feature = "history")]
#[test]
fn optional_single_history_dates_rejects_multiple_rows() {
    let rows = vec![
        HistoryDates::try_new(d("2013-03-25"), d("2026-03-06")).expect("valid dates"),
        HistoryDates::try_new(d("2011-01-01"), d("2012-01-01")).expect("valid dates"),
    ];
    let err = optional_single_history_dates("history/.../dates.json", rows)
        .expect_err("multiple rows must be rejected");
    match err {
        MoexError::UnexpectedHistoryDatesRows {
            endpoint,
            row_count,
        } => {
            assert_eq!(endpoint.as_ref(), "history/.../dates.json");
            assert_eq!(row_count, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn append_pagination_to_url_sets_start_and_limit() {
    let mut url = Url::parse("https://example.test/iss/engines.json").expect("valid URL");
    let pagination = Pagination::default()
        .with_start(200)
        .with_limit(NonZeroU32::new(5_000).expect("non-zero limit"));

    append_pagination_to_url(&mut url, pagination);

    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == START_PARAM && v == "200")
    );
    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == LIMIT_PARAM && v == "5000")
    );
}

#[test]
fn append_candle_query_to_url_uses_datetime_format() {
    let mut url = Url::parse("https://example.test/iss/candles.json").expect("valid URL");
    let query = CandleQuery::default()
        .with_from(dt("2026-03-01 10:15:30"))
        .expect("valid from")
        .with_till(dt("2026-03-06 18:45:00"))
        .expect("valid till");

    append_candle_query_to_url(&mut url, query);

    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == FROM_PARAM && v == "2026-03-01 10:15:30")
    );
    assert!(
        query_pairs
            .iter()
            .any(|(k, v)| k == TILL_PARAM && v == "2026-03-06 18:45:00")
    );
}

#[test]
fn append_pagination_to_url_skips_absent_values() {
    let mut url = Url::parse("https://example.test/iss/engines.json").expect("valid URL");

    append_pagination_to_url(&mut url, Pagination::default());

    assert_eq!(url.query(), None);
}

#[test]
fn collect_paginated_combines_pages_until_short_page() {
    let page_limit = NonZeroU32::new(2).expect("non-zero limit");
    let mut calls = 0;
    let items = BlockingMoexClient::collect_paginated(
        "test/endpoint",
        page_limit,
        RepeatPagePolicy::Error,
        |pagination| {
            calls += 1;
            match pagination.start {
                Some(0) => Ok(vec![10_u64, 11_u64]),
                Some(2) => Ok(vec![12_u64]),
                Some(_) => Ok(Vec::new()),
                None => unreachable!("collect_paginated always sets start"),
            }
        },
        |item| *item,
    )
    .expect("pagination should succeed");

    assert_eq!(calls, 2);
    assert_eq!(items, vec![10_u64, 11_u64, 12_u64]);
}

#[test]
fn collect_paginated_reports_stuck_pages() {
    let page_limit = NonZeroU32::new(2).expect("non-zero limit");
    let err = BlockingMoexClient::collect_paginated(
        "test/endpoint",
        page_limit,
        RepeatPagePolicy::Error,
        |_| Ok(vec![10_u64, 11_u64]),
        |item| *item,
    )
    .expect_err("must detect pagination loop");

    match err {
        MoexError::PaginationStuck {
            endpoint,
            start,
            limit,
        } => {
            assert_eq!(endpoint.as_ref(), "test/endpoint");
            assert_eq!(start, 2);
            assert_eq!(limit, 2);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn collect_paginated_repeated_page_returns_error_instead_of_partial_data() {
    let page_limit = NonZeroU32::new(2).expect("non-zero limit");
    let mut calls = 0;
    let err = BlockingMoexClient::collect_paginated(
        "test/endpoint",
        page_limit,
        RepeatPagePolicy::Error,
        |pagination| {
            calls += 1;
            match pagination.start {
                Some(0) => Ok(vec![10_u64, 11_u64]),
                Some(2) => Ok(vec![10_u64, 11_u64]),
                Some(_) => Ok(Vec::new()),
                None => unreachable!("collect_paginated always sets start"),
            }
        },
        |item| *item,
    )
    .expect_err("repeated page must fail with explicit error");

    assert_eq!(calls, 2);
    assert!(matches!(
        err,
        MoexError::PaginationStuck {
            endpoint: _,
            start: 2,
            limit: 2
        }
    ));
}

#[test]
fn looks_like_json_payload_accepts_json_content_type() {
    assert!(looks_like_json_payload(
        Some("application/json; charset=utf-8"),
        "<html>not-json</html>"
    ));
}

#[test]
fn looks_like_json_payload_accepts_json_body_without_header() {
    assert!(looks_like_json_payload(None, "  {\"ok\":true}"));
    assert!(looks_like_json_payload(Some("text/plain"), "\n [1,2,3]"));
}

#[test]
fn looks_like_json_payload_rejects_html_payload() {
    assert!(!looks_like_json_payload(
        Some("text/html; charset=utf-8"),
        "<!DOCTYPE html><html></html>"
    ));
}

#[test]
fn truncate_prefix_limits_length() {
    let text = "abcdef";
    assert_eq!(truncate_prefix(text, 4).as_ref(), "abcd");
    assert_eq!(truncate_prefix(text, 10).as_ref(), "abcdef");
}

#[test]
fn builder_constructs_client() {
    let client = BlockingMoexClient::builder()
        .metadata(true)
        .build()
        .expect("valid default builder");
    assert_eq!(client.rate_limit(), None);
}

#[test]
fn constructors_without_semantic_bool_build_clients() {
    let _default = BlockingMoexClient::new().expect("default constructor must work");
    let _with_metadata =
        BlockingMoexClient::new_with_metadata().expect("metadata constructor must work");

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_client =
        BlockingMoexClient::with_client(blocking_http).expect("constructor with client");

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_client_metadata = BlockingMoexClient::with_client_with_metadata(blocking_http)
        .expect("constructor with client and metadata");

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_base_url = BlockingMoexClient::with_base_url(
        blocking_http,
        Url::parse(BASE_URL).expect("valid base url"),
    );

    let blocking_http = reqwest::blocking::Client::builder()
        .build()
        .expect("valid blocking client");
    let _with_base_url_metadata = BlockingMoexClient::with_base_url_with_metadata(
        blocking_http,
        Url::parse(BASE_URL).expect("valid base url"),
    );
}

#[test]
fn builder_rate_limit_is_propagated_to_client() {
    let limit = RateLimit::every(Duration::from_millis(250));
    let client = BlockingMoexClient::builder()
        .rate_limit(limit)
        .build()
        .expect("builder with rate limit must construct client");

    assert_eq!(client.rate_limit(), Some(limit));
}

#[test]
fn builder_user_agent_from_crate_constructs_client() {
    let _client = BlockingMoexClient::builder()
        .user_agent_from_crate()
        .build()
        .expect("user-agent from crate must be valid");
}

#[test]
fn builder_proxy_constructs_client() {
    let proxy = reqwest::Proxy::all("http://127.0.0.1:8090").expect("valid proxy URL");
    let _client = BlockingMoexClient::builder()
        .proxy(proxy)
        .build()
        .expect("builder with proxy must construct client");
}

#[test]
fn builder_no_proxy_constructs_client() {
    let _client = BlockingMoexClient::builder()
        .no_proxy()
        .build()
        .expect("builder with no_proxy must construct client");
}

#[cfg(feature = "async")]
#[test]
fn async_builder_constructs_client() {
    let client = AsyncMoexClient::builder()
        .metadata(true)
        .build()
        .expect("valid async default builder");
    assert_eq!(client.rate_limit(), None);
}

#[cfg(feature = "async")]
#[test]
fn async_constructors_without_semantic_bool_build_clients() {
    let _default = AsyncMoexClient::new().expect("default constructor must work");
    let _with_metadata = AsyncMoexClient::new_with_metadata().expect("metadata constructor");

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_client = AsyncMoexClient::with_client(async_http).expect("constructor with client");

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_client_metadata =
        AsyncMoexClient::with_client_with_metadata(async_http).expect("with metadata");

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_base_url =
        AsyncMoexClient::with_base_url(async_http, Url::parse(BASE_URL).expect("valid base url"));

    let async_http = reqwest::Client::builder()
        .build()
        .expect("valid async client");
    let _with_base_url_metadata = AsyncMoexClient::with_base_url_with_metadata(
        async_http,
        Url::parse(BASE_URL).expect("valid base url"),
    );
}

#[cfg(feature = "async")]
#[test]
fn async_builder_rate_limit_without_sleep_returns_error() {
    let err = match AsyncMoexClient::builder()
        .rate_limit(RateLimit::every(Duration::from_millis(250)))
        .build()
    {
        Ok(_) => panic!("async rate limit without sleep must fail on build"),
        Err(err) => err,
    };

    assert!(matches!(err, MoexError::MissingAsyncRateLimitSleep));
}

#[cfg(feature = "async")]
#[test]
fn async_builder_rate_limit_with_sleep_constructs_client() {
    let limit = RateLimit::every(Duration::from_millis(250));
    let client = AsyncMoexClient::builder()
        .rate_limit(limit)
        .rate_limit_sleep(|_delay| std::future::ready(()))
        .build()
        .expect("async builder with rate limit sleep must construct client");

    assert_eq!(client.rate_limit(), Some(limit));
}

#[cfg(feature = "async")]
#[test]
fn async_builder_user_agent_from_crate_constructs_client() {
    let _client = AsyncMoexClient::builder()
        .user_agent_from_crate()
        .build()
        .expect("async user-agent from crate must be valid");
}

#[cfg(feature = "async")]
#[test]
fn async_builder_proxy_constructs_client() {
    let proxy = reqwest::Proxy::all("http://127.0.0.1:8090").expect("valid proxy URL");
    let _client = AsyncMoexClient::builder()
        .proxy(proxy)
        .build()
        .expect("async builder with proxy must construct client");
}

#[cfg(feature = "async")]
#[test]
fn async_builder_no_proxy_constructs_client() {
    let _client = AsyncMoexClient::builder()
        .no_proxy()
        .build()
        .expect("async builder with no_proxy must construct client");
}

#[test]
fn builder_propagates_http_client_build_errors() {
    let err = match BlockingMoexClient::builder()
        .user_agent("invalid\nagent")
        .build()
    {
        Ok(_) => panic!("invalid user-agent must fail on build"),
        Err(err) => err,
    };

    assert!(
        matches!(err, MoexError::BuildHttpClient { .. }),
        "unexpected error variant: {err:?}"
    );
}

#[cfg(feature = "async")]
#[test]
fn async_builder_propagates_http_client_build_errors() {
    let err = match AsyncMoexClient::builder()
        .user_agent("invalid\nagent")
        .build()
    {
        Ok(_) => panic!("invalid user-agent must fail on async build"),
        Err(err) => err,
    };

    assert!(
        matches!(err, MoexError::BuildHttpClient { .. }),
        "unexpected error variant: {err:?}"
    );
}

#[test]
fn owned_scopes_accept_try_into_inputs_and_shortcuts() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");
    let page_limit = NonZeroU32::new(1000).expect("non-zero page limit");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let market_scope = client
        .stock()
        .expect("valid stock shortcut")
        .shares()
        .expect("valid shares shortcut");

    let board_scope = market_scope.clone().board(&board).expect("valid board");
    assert_eq!(board_scope.engine().as_str(), "stock");
    assert_eq!(board_scope.market().as_str(), "shares");
    assert_eq!(board_scope.board().as_str(), "TQBR");

    let market_security_scope = market_scope
        .security("SBER")
        .expect("valid market security");
    assert_eq!(market_security_scope.security().as_str(), "SBER");

    let security_scope = board_scope
        .clone()
        .security("SBER")
        .expect("valid security");
    assert_eq!(security_scope.security().as_str(), "SBER");

    let _securities_pages = board_scope.securities_pages(page_limit);
    let _trades_pages = security_scope.trades_pages(page_limit);

    let index_scope = client.index("IMOEX").expect("valid indexid");
    assert_eq!(index_scope.indexid().as_str(), "IMOEX");
    let _analytics_pages = index_scope.analytics_pages(page_limit);

    let security_resource_scope = client.security("SBER").expect("valid secid");
    assert_eq!(security_resource_scope.secid().as_str(), "SBER");
}

#[test]
fn owned_scopes_fail_fast_on_invalid_identifiers() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");

    let invalid_engine = client.engine("");
    assert!(
        matches!(
            invalid_engine,
            Err(crate::models::ParseEngineNameError::Empty)
        ),
        "unexpected engine parse result"
    );

    let invalid_board = client
        .stock()
        .expect("valid stock shortcut")
        .shares()
        .expect("valid shares shortcut")
        .board(" ");
    assert!(
        matches!(invalid_board, Err(crate::models::ParseBoardIdError::Empty)),
        "unexpected board parse result"
    );

    let invalid_security = client.security(" ");
    assert!(
        matches!(invalid_security, Err(crate::models::ParseSecIdError::Empty)),
        "unexpected security parse result"
    );
}

#[cfg(feature = "async")]
#[test]
fn async_owned_scopes_accept_try_into_inputs_and_shortcuts() {
    let client = AsyncMoexClient::builder()
        .build()
        .expect("valid async default builder");
    let page_limit = NonZeroU32::new(1000).expect("non-zero page limit");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let market_scope = client
        .stock()
        .expect("valid stock shortcut")
        .shares()
        .expect("valid shares shortcut");

    let board_scope = market_scope.clone().board(&board).expect("valid board");
    assert_eq!(board_scope.engine().as_str(), "stock");
    assert_eq!(board_scope.market().as_str(), "shares");
    assert_eq!(board_scope.board().as_str(), "TQBR");

    let market_security_scope = market_scope
        .security("SBER")
        .expect("valid market security");
    assert_eq!(market_security_scope.security().as_str(), "SBER");

    let security_scope = board_scope
        .clone()
        .security("SBER")
        .expect("valid security");
    assert_eq!(security_scope.security().as_str(), "SBER");

    let _securities_pages = board_scope.securities_pages(page_limit);
    let _trades_pages = security_scope.trades_pages(page_limit);

    let index_scope = client.index("IMOEX").expect("valid indexid");
    assert_eq!(index_scope.indexid().as_str(), "IMOEX");
    let _analytics_pages = index_scope.analytics_pages(page_limit);

    let security_resource_scope = client.security("SBER").expect("valid secid");
    assert_eq!(security_resource_scope.secid().as_str(), "SBER");
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

#[test]
fn normalize_raw_endpoint_path_supports_common_forms() {
    assert_eq!(
        normalize_raw_endpoint_path(Some("engines"))
            .expect("must normalize")
            .as_ref(),
        "engines.json"
    );
    assert_eq!(
        normalize_raw_endpoint_path(Some("engines.json"))
            .expect("must keep explicit extension")
            .as_ref(),
        "engines.json"
    );
    assert_eq!(
        normalize_raw_endpoint_path(Some("/iss/engines"))
            .expect("must trim /iss/ prefix")
            .as_ref(),
        "engines.json"
    );
}

#[test]
fn normalize_raw_endpoint_path_rejects_missing_or_invalid_path() {
    let missing = normalize_raw_endpoint_path(None).expect_err("path is required");
    assert!(matches!(missing, MoexError::MissingRawPath));

    let empty = normalize_raw_endpoint_path(Some("  ")).expect_err("empty path must fail");
    assert!(matches!(empty, MoexError::InvalidRawPath { .. }));

    let with_query = normalize_raw_endpoint_path(Some("engines?iss.meta=on"))
        .expect_err("query string in path must fail");
    assert!(matches!(with_query, MoexError::InvalidRawPath { .. }));
}

#[test]
fn raw_builder_requires_path_before_send() {
    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");

    let err = client
        .raw()
        .send_payload()
        .expect_err("raw request without path must fail");
    assert!(matches!(err, MoexError::MissingRawPath));
}

#[test]
fn iss_request_options_and_raw_response_helpers_work() {
    let options = IssRequestOptions::new()
        .metadata(IssToggle::On)
        .data(IssToggle::Off)
        .version(IssToggle::On)
        .json("extended");
    assert_eq!(options.metadata_value(), Some(IssToggle::On));
    assert_eq!(options.data_value(), Some(IssToggle::Off));
    assert_eq!(options.version_value(), Some(IssToggle::On));
    assert_eq!(options.json_value(), Some("extended"));

    let client = BlockingMoexClient::builder()
        .build()
        .expect("valid default builder");
    let _raw = client
        .raw()
        .path("engines")
        .options(options)
        .metadata(IssToggle::On)
        .data(IssToggle::On)
        .version(IssToggle::Off)
        .json("extended");

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let response = RawIssResponse::new(StatusCode::OK, headers.clone(), "{\"ok\":true}".into());
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .expect("content-type header")
            .to_str()
            .expect("header to str"),
        "application/json"
    );
    assert_eq!(response.body(), "{\"ok\":true}");

    let (status, returned_headers, body) = response.into_parts();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        returned_headers
            .get(CONTENT_TYPE)
            .expect("content-type header")
            .to_str()
            .expect("header to str"),
        "application/json"
    );
    assert_eq!(body, "{\"ok\":true}");
}

#[test]
fn moex_error_helpers_detect_retryable_status_and_body_prefix() {
    let err = MoexError::HttpStatus {
        endpoint: "engines.json".into(),
        status: StatusCode::TOO_MANY_REQUESTS,
        content_type: Some("application/json".into()),
        body_prefix: "{\"error\":\"rate limit\"}".into(),
    };

    assert!(err.is_retryable());
    assert_eq!(err.status_code(), Some(StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(
        err.response_body_prefix(),
        Some("{\"error\":\"rate limit\"}")
    );
}

#[test]
fn moex_error_helpers_mark_non_retryable_client_status() {
    let err = MoexError::HttpStatus {
        endpoint: "engines.json".into(),
        status: StatusCode::BAD_REQUEST,
        content_type: Some("application/json".into()),
        body_prefix: "{\"error\":\"bad request\"}".into(),
    };

    assert!(!err.is_retryable());
    assert_eq!(err.status_code(), Some(StatusCode::BAD_REQUEST));
}

#[test]
fn moex_error_helpers_return_body_prefix_for_non_json_payload() {
    let err = MoexError::NonJsonPayload {
        endpoint: "engines.json".into(),
        content_type: Some("text/html".into()),
        body_prefix: "<!doctype html>".into(),
    };

    assert_eq!(err.response_body_prefix(), Some("<!doctype html>"));
    assert_eq!(err.status_code(), None);
    assert!(!err.is_retryable());
}

#[test]
fn with_retry_retries_retryable_errors_until_success() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(3).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let value = with_retry(policy, || {
        attempts += 1;
        if attempts < 3 {
            return Err(MoexError::HttpStatus {
                endpoint: "engines.json".into(),
                status: StatusCode::TOO_MANY_REQUESTS,
                content_type: Some("application/json".into()),
                body_prefix: "{\"error\":\"rate limit\"}".into(),
            });
        }
        Ok(42_u32)
    })
    .expect("third attempt must succeed");

    assert_eq!(value, 42_u32);
    assert_eq!(attempts, 3);
}

#[test]
fn retry_policy_default_matches_example_values() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_attempts().get(), 3);
    assert_eq!(policy.delay(), Duration::from_millis(400));
}

#[test]
fn with_retry_stops_on_non_retryable_error_without_extra_attempts() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(5).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let err = with_retry::<(), _>(policy, || {
        attempts += 1;
        Err(MoexError::HttpStatus {
            endpoint: "engines.json".into(),
            status: StatusCode::BAD_REQUEST,
            content_type: Some("application/json".into()),
            body_prefix: "{\"error\":\"bad request\"}".into(),
        })
    })
    .expect_err("non-retryable error must be returned immediately");

    assert!(matches!(
        err,
        MoexError::HttpStatus {
            status: StatusCode::BAD_REQUEST,
            ..
        }
    ));
    assert_eq!(attempts, 1);
}

#[test]
fn with_retry_stops_after_max_attempts_on_retryable_error() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(2).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let err = with_retry::<(), _>(policy, || {
        attempts += 1;
        Err(MoexError::HttpStatus {
            endpoint: "engines.json".into(),
            status: StatusCode::SERVICE_UNAVAILABLE,
            content_type: Some("application/json".into()),
            body_prefix: "{\"error\":\"temporary unavailable\"}".into(),
        })
    })
    .expect_err("retryable error must stop after max attempts");

    assert!(matches!(
        err,
        MoexError::HttpStatus {
            status: StatusCode::SERVICE_UNAVAILABLE,
            ..
        }
    ));
    assert_eq!(attempts, 2);
}

#[test]
fn rate_limit_per_second_rounds_up_interval() {
    let limit = RateLimit::per_second(NonZeroU32::new(3).expect("non-zero requests per second"));
    assert_eq!(limit.min_interval(), Duration::from_nanos(333_333_334));
}

#[test]
fn rate_limiter_reserve_delay_at_tracks_slots() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_millis(100)));
    let start = Instant::now();

    assert_eq!(limiter.reserve_delay_at(start), Duration::ZERO);
    assert_eq!(limiter.reserve_delay_at(start), Duration::from_millis(100));
    assert_eq!(
        limiter.reserve_delay_at(start + Duration::from_millis(250)),
        Duration::ZERO
    );
    assert_eq!(
        limiter.reserve_delay_at(start + Duration::from_millis(300)),
        Duration::from_millis(50)
    );
}

#[test]
fn with_rate_limit_executes_action_and_returns_value() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::ZERO));
    let value = with_rate_limit(&mut limiter, || 42_u32);
    assert_eq!(value, 42_u32);
}

#[cfg(feature = "async")]
fn block_on_immediate<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut future = pin!(future);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("test future unexpectedly returned Poll::Pending"),
    }
}

#[cfg(feature = "async")]
#[test]
fn with_rate_limit_async_skips_sleep_on_first_slot() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_secs(1)));
    let mut sleep_calls = 0_u32;

    let value = block_on_immediate(with_rate_limit_async(
        &mut limiter,
        || std::future::ready(7_u32),
        |_delay| {
            sleep_calls += 1;
            std::future::ready(())
        },
    ));

    assert_eq!(value, 7_u32);
    assert_eq!(sleep_calls, 0);
}

#[cfg(feature = "async")]
#[test]
fn with_rate_limit_async_uses_injected_sleep_when_delay_is_reserved() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_secs(1)));
    limiter.next_allowed_at = Some(Instant::now() + Duration::from_secs(1));
    let mut slept = Vec::new();

    let value = block_on_immediate(with_rate_limit_async(
        &mut limiter,
        || std::future::ready(9_u32),
        |delay| {
            slept.push(delay);
            std::future::ready(())
        },
    ));

    assert_eq!(value, 9_u32);
    assert_eq!(slept.len(), 1);
    assert!(slept[0] > Duration::ZERO);
    assert!(slept[0] <= Duration::from_secs(1));
}

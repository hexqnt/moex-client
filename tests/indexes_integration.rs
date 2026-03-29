#![cfg(feature = "blocking")]

use moex_client::{MoexError, decode};

#[test]
fn parse_typical_iss_indexes_payload() {
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

    let indexes = decode::indexes_json(payload).expect("valid payload");

    assert_eq!(indexes.len(), 2);
    assert_eq!(indexes[0].id().as_str(), "IMOEX");
    assert_eq!(indexes[0].short_name(), "Индекс МосБиржи");
    assert_eq!(indexes[1].id().as_str(), "RTSI");
    assert_eq!(indexes[1].till(), None);
}

#[test]
fn parse_payload_surfaces_invalid_row_index() {
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

    let err = decode::indexes_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidIndex { row: 1, .. }));
}

#[test]
fn parse_typical_iss_index_analytics_payload() {
    let payload = r#"
    {
        "analytics": {
            "columns": ["indexid", "tradedate", "ticker", "shortnames", "secids", "weight", "tradingsession", "trade_session_date"],
            "data": [
                ["IMOEX", "2026-03-06", "SBER", "Сбербанк", "SBER", 14.56, 3, "2026-03-06"],
                ["IMOEX", "2026-03-06", "GAZP", "ГАЗПРОМ ао", "GAZP", 9.8, 3, "2026-03-06"]
            ]
        }
    }
    "#;

    let analytics = decode::index_analytics_json(payload).expect("valid payload");

    assert_eq!(analytics.len(), 2);
    assert_eq!(analytics[0].indexid().as_str(), "IMOEX");
    assert_eq!(analytics[0].ticker().as_str(), "SBER");
    assert_eq!(analytics[0].weight(), 14.56);
    assert_eq!(analytics[1].secid().as_str(), "GAZP");
}

#[test]
fn parse_index_analytics_payload_surfaces_invalid_row_index() {
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

    let err = decode::index_analytics_json(payload).expect_err("invalid second row");
    assert!(matches!(
        err,
        MoexError::InvalidIndexAnalytics { row: 1, .. }
    ));
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

    let engines = decode::engines_json(payload).expect("valid payload");

    assert_eq!(engines.len(), 2);
    assert_eq!(engines[0].id().get(), 1);
    assert_eq!(engines[0].name().as_str(), "stock");
    assert_eq!(engines[1].id().get(), 4);
}

#[test]
fn parse_engines_payload_surfaces_invalid_row_index() {
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

    let err = decode::engines_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidEngine { row: 1, .. }));
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

    let markets = decode::markets_json(payload).expect("valid payload");

    assert_eq!(markets.len(), 2);
    assert_eq!(markets[0].id().get(), 5);
    assert_eq!(markets[0].name().as_str(), "index");
}

#[test]
fn parse_markets_payload_surfaces_invalid_row_index() {
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

    let err = decode::markets_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidMarket { row: 1, .. }));
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

    let boards = decode::boards_json(payload).expect("valid payload");

    assert_eq!(boards.len(), 2);
    assert_eq!(boards[0].id(), 95);
    assert_eq!(boards[0].boardid().as_str(), "EQCC");
    assert!(!boards[0].is_traded());
}

#[test]
fn parse_boards_payload_surfaces_invalid_row_index() {
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

    let err = decode::boards_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidBoard { row: 1, .. }));
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

    let securities = decode::securities_json(payload).expect("valid payload");

    assert_eq!(securities.len(), 2);
    assert_eq!(securities[0].secid().as_str(), "ABIO");
    assert_eq!(securities[1].status(), "N");
}

#[test]
fn parse_securities_payload_surfaces_invalid_row_index() {
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

    let err = decode::securities_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidSecurity { row: 1, .. }));
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

    let levels = decode::orderbook_json(payload).expect("valid payload");

    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0].buy_sell().as_str(), "B");
    assert_eq!(levels[0].price(), 314.79);
    assert_eq!(levels[1].buy_sell().as_str(), "S");
}

#[test]
fn parse_orderbook_payload_surfaces_invalid_row_index() {
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

    let err = decode::orderbook_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidOrderbook { row: 1, .. }));
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

    let candles = decode::candles_json(payload).expect("valid payload");

    assert_eq!(candles.len(), 2);
    assert_eq!(candles[0].close(), Some(109.2));
    assert_eq!(candles[1].volume(), Some(35_092_029));
}

#[test]
fn parse_candles_payload_surfaces_invalid_row_index() {
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

    let err = decode::candles_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidCandle { row: 1, .. }));
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

    let trades = decode::trades_json(payload).expect("valid payload");

    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].tradeno(), 15_780_983_820);
    assert_eq!(trades[0].price(), Some(314.8));
}

#[test]
fn parse_trades_payload_surfaces_invalid_row_index() {
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

    let err = decode::trades_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidTrade { row: 1, .. }));
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
fn parse_sitenews_payload_surfaces_invalid_row_index() {
    let payload = r#"
    {
        "sitenews": {
            "columns": ["id", "tag", "title", "published_at", "modified_at"],
            "data": [
                [98236, "site", "О дополнительных условиях проведения торгов", "2026-03-06 19:08:57", "2026-03-06 19:08:57"],
                [98235, "site", "   ", "2026-03-06 18:00:00", "2026-03-06 18:00:00"]
            ]
        }
    }
    "#;

    let err = decode::sitenews_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidSiteNews { row: 1, .. }));
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
fn parse_events_payload_surfaces_invalid_row_index() {
    let payload = r#"
    {
        "events": {
            "columns": ["id", "tag", "title", "from", "modified_at"],
            "data": [
                [77, "site", "Технические работы", "2026-03-07 10:00:00", "2026-03-06 22:00:00"],
                [78, "   ", "Событие без from", null, "2026-03-06 21:00:00"]
            ]
        }
    }
    "#;

    let err = decode::events_json(payload).expect_err("invalid second row");
    assert!(matches!(err, MoexError::InvalidEvent { row: 1, .. }));
}

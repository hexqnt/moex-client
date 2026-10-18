use chrono::NaiveDate;
use moex_client::{MoexError, decode, models::CandleInterval};

fn d(input: &str) -> NaiveDate {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").unwrap()
}

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
    assert_eq!(indexes[0].from(), Some(d("2001-01-03")));
    assert_eq!(indexes[0].till(), Some(d("2026-03-05")));
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
    assert_eq!(analytics[1].trade_session_date(), d("2026-03-06"));
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
    assert_eq!(levels[0].quantity(), 1500);
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
                "columns": ["BOARDID", "TRADEDATE", "SECID", "NUMTRADES", "VALUE", "OPEN", "LOW", "HIGH", "CLOSE", "VOLUME", "DURATION", "YIELD"],
                "data": [
                    ["TQBR", "2026-03-05", "SBER", 120345, 123456789.5, 314.0, 310.0, 315.2, 314.8, 3900000, 1733, 15.49],
                    ["TQBR", "2026-03-06", "SBER", 118000, 118000000.0, 314.8, 311.5, 316.0, 315.3, 3700000, 1727, 15.64]
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
    assert_eq!(history[0].duration_days(), Some(1_733.0));
    assert_eq!(history[0].yield_percent(), Some(15.49));
}

#[cfg(feature = "history")]
#[test]
fn history_decoder_maps_market_specific_fields_by_column_name() {
    let payload = r#"
        {
            "history": {
                "columns": ["YIELD", "SECID", "DURATION", "TRADEDATE", "BOARDID"],
                "data": [
                    [15.49, "RGBITR", 1733.5, "2026-03-05", "SNDX"]
                ]
            }
        }
        "#;

    let history = decode::history_json(payload).expect("valid payload");

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].boardid().as_str(), "SNDX");
    assert_eq!(history[0].secid().as_str(), "RGBITR");
    assert_eq!(history[0].tradedate(), d("2026-03-05"));
    assert_eq!(history[0].duration_days(), Some(1_733.5));
    assert_eq!(history[0].yield_percent(), Some(15.49));
}

#[cfg(feature = "history")]
#[test]
fn history_decoder_accepts_payload_without_market_specific_fields() {
    let payload = r#"
        {
            "history": {
                "columns": ["BOARDID", "TRADEDATE", "SECID", "CLOSE"],
                "data": [
                    ["TQBR", "2026-03-05", "SBER", 314.8]
                ]
            }
        }
        "#;

    let history = decode::history_json(payload).expect("valid payload");

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].close(), Some(314.8));
    assert_eq!(history[0].duration_days(), None);
    assert_eq!(history[0].yield_percent(), None);
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

use std::num::NonZeroU32;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::*;

fn d(input: &str) -> NaiveDate {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").unwrap()
}

fn dt(input: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap()
}

fn t(input: &str) -> NaiveTime {
    NaiveTime::parse_from_str(input, "%H:%M:%S").unwrap()
}

#[test]
fn empty_till_in_wire_row_maps_to_none() {
    let row: IndexRow = serde_json::from_str(r#"["IMOEX","Индекс МосБиржи","2001-01-03",""]"#)
        .expect("valid wire row");
    let index = Index::try_from(row).expect("valid domain index");
    assert_eq!(index.till(), None);
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

#[cfg(feature = "history")]
#[test]
fn parses_history_dates_row() {
    let row: HistoryDatesRow =
        serde_json::from_str(r#"["2013-03-25","2026-03-06"]"#).expect("valid history dates row");
    let dates = HistoryDates::try_from(row).expect("valid history dates");
    assert_eq!(dates.from(), d("2013-03-25"));
    assert_eq!(dates.till(), d("2026-03-06"));
}

#[cfg(feature = "history")]
#[test]
fn rejects_invalid_history_dates_range() {
    let err = HistoryDates::try_new(d("2026-03-07"), d("2026-03-06"))
        .expect_err("must reject invalid history dates range");
    assert!(matches!(
        err,
        ParseHistoryDatesError::InvalidDateRange { .. }
    ));
}

#[cfg(feature = "history")]
#[test]
fn parses_history_row() {
    let row: HistoryRow = serde_json::from_str(
        r#"["TQBR","2026-03-06","SBER",120345,123456789.5,314.0,310.0,315.2,314.8,3900000]"#,
    )
    .expect("valid history row");
    let history = HistoryRecord::try_from(row).expect("valid history");
    assert_eq!(history.boardid().as_str(), "TQBR");
    assert_eq!(history.tradedate(), d("2026-03-06"));
    assert_eq!(history.secid().as_str(), "SBER");
    assert_eq!(history.numtrades(), Some(120_345));
    assert_eq!(history.close(), Some(314.8));
    assert_eq!(history.volume(), Some(3_900_000));
}

#[cfg(feature = "history")]
#[test]
fn rejects_negative_history_volume() {
    let row: HistoryRow = serde_json::from_str(
        r#"["TQBR","2026-03-06","SBER",120345,123456789.5,314.0,310.0,315.2,314.8,-1]"#,
    )
    .expect("valid history row");
    let err = HistoryRecord::try_from(row).expect_err("must reject negative history volume");
    assert!(matches!(err, ParseHistoryRecordError::NegativeVolume(-1)));
}

#[test]
fn parses_turnover_row() {
    let row: TurnoverRow = serde_json::from_str(
        r#"["shares",1,24573900000.5,318000000.1,120345,"2026-03-06 23:50:23","Рынок акций"]"#,
    )
    .expect("valid turnover row");
    let turnover = Turnover::try_from(row).expect("valid turnover");
    assert_eq!(turnover.name(), "shares");
    assert_eq!(turnover.id(), 1);
    assert_eq!(turnover.numtrades(), Some(120_345));
    assert_eq!(turnover.updatetime(), dt("2026-03-06 23:50:23"));
    assert_eq!(turnover.title(), "Рынок акций");
}

#[test]
fn rejects_negative_turnover_numtrades() {
    let row: TurnoverRow = serde_json::from_str(
        r#"["shares",1,24573900000.5,318000000.1,-1,"2026-03-06 23:50:23","Рынок акций"]"#,
    )
    .expect("valid turnover row");
    let err = Turnover::try_from(row).expect_err("must reject negative turnover numtrades");
    assert!(matches!(err, ParseTurnoverError::NegativeNumTrades(-1)));
}

#[test]
fn parses_secstat_row() {
    let row: SecStatRow = serde_json::from_str(
        r#"["SBER","TQBR",12500000,3950000000.5,314.79,314.8,314.8,314.79,313.0,312.4,315.0,314.8,157809,314.55]"#,
    )
    .expect("valid secstats row");
    let stat = SecStat::try_from(row).expect("valid secstats");
    assert_eq!(stat.secid().as_str(), "SBER");
    assert_eq!(stat.boardid().as_str(), "TQBR");
    assert_eq!(stat.voltoday(), Some(12_500_000));
    assert_eq!(stat.numtrades(), Some(157_809));
    assert_eq!(stat.last(), Some(314.8));
}

#[test]
fn rejects_negative_secstat_voltoday() {
    let row: SecStatRow = serde_json::from_str(
        r#"["SBER","TQBR",-1,3950000000.5,314.79,314.8,314.8,314.79,313.0,312.4,315.0,314.8,157809,314.55]"#,
    )
    .expect("valid secstats row");
    let err = SecStat::try_from(row).expect_err("must reject negative secstats volume");
    assert!(matches!(err, ParseSecStatError::NegativeVolToday(-1)));
}

#[cfg(feature = "news")]
#[test]
fn parses_sitenews_row() {
    let row: SiteNewsRow = serde_json::from_str(
        r#"[98236,"site","О дополнительных условиях проведения торгов","2026-03-06 19:08:57","2026-03-06 19:08:57"]"#,
    )
    .expect("valid sitenews row");
    let news = SiteNews::try_from(row).expect("valid sitenews");
    assert_eq!(news.id(), 98_236);
    assert_eq!(news.tag(), "site");
    assert_eq!(news.title(), "О дополнительных условиях проведения торгов");
    assert_eq!(news.published_at(), dt("2026-03-06 19:08:57"));
}

#[cfg(feature = "news")]
#[test]
fn rejects_sitenews_with_empty_title() {
    let row: SiteNewsRow =
        serde_json::from_str(r#"[98236,"site","  ","2026-03-06 19:08:57","2026-03-06 19:08:57"]"#)
            .expect("valid sitenews row");
    let err = SiteNews::try_from(row).expect_err("must reject empty sitenews title");
    assert!(matches!(err, ParseSiteNewsError::EmptyTitle));
}

#[cfg(feature = "news")]
#[test]
fn parses_event_row_with_optional_from() {
    let row: EventRow = serde_json::from_str(
        r#"[77,"site","Технические работы","2026-03-07 10:00:00","2026-03-06 22:00:00"]"#,
    )
    .expect("valid events row");
    let event = Event::try_from(row).expect("valid event");
    assert_eq!(event.id(), 77);
    assert_eq!(event.tag(), "site");
    assert_eq!(event.from(), Some(dt("2026-03-07 10:00:00")));
    assert_eq!(event.modified_at(), dt("2026-03-06 22:00:00"));
}

#[cfg(feature = "news")]
#[test]
fn parses_event_row_with_null_from() {
    let row: EventRow =
        serde_json::from_str(r#"[78,"site","Событие без from",null,"2026-03-06 22:00:00"]"#)
            .expect("valid events row");
    let event = Event::try_from(row).expect("valid event");
    assert_eq!(event.id(), 78);
    assert_eq!(event.from(), None);
}

#[cfg(feature = "news")]
#[test]
fn rejects_event_with_empty_tag() {
    let row: EventRow =
        serde_json::from_str(r#"[77,"  ","Технические работы",null,"2026-03-06 22:00:00"]"#)
            .expect("valid events row");
    let err = Event::try_from(row).expect_err("must reject empty event tag");
    assert!(matches!(err, ParseEventError::EmptyTag));
}

#[test]
fn actual_indexes_pick_latest_till() {
    let older = Index::try_new(
        "RTSI".to_string(),
        "Индекс РТС".to_string(),
        Some(d("2001-01-03")),
        Some(d("2026-03-04")),
    )
    .unwrap();
    let latest_a = Index::try_new(
        "IMOEX".to_string(),
        "Индекс МосБиржи".to_string(),
        Some(d("2001-01-03")),
        Some(d("2026-03-05")),
    )
    .unwrap();
    let latest_b = Index::try_new(
        "MOEXSM".to_string(),
        "Индекс МосБиржи SMID".to_string(),
        Some(d("2001-01-03")),
        Some(d("2026-03-05")),
    )
    .unwrap();
    let indexes = vec![older, latest_a.clone(), latest_b.clone()];

    let actual_ids: Vec<&str> = actual_indexes(&indexes)
        .map(|index| index.id().as_str())
        .collect();
    assert_eq!(
        actual_ids,
        vec![latest_a.id().as_str(), latest_b.id().as_str()]
    );
}

#[test]
fn indexes_ext_keeps_latest_till() {
    let older = Index::try_new(
        "RTSI".to_string(),
        "Индекс РТС".to_string(),
        Some(d("2001-01-03")),
        Some(d("2026-03-04")),
    )
    .unwrap();
    let latest = Index::try_new(
        "IMOEX".to_string(),
        "Индекс МосБиржи".to_string(),
        Some(d("2001-01-03")),
        Some(d("2026-03-05")),
    )
    .unwrap();

    let filtered = vec![older, latest.clone()].into_actual_by_till();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id().as_str(), latest.id().as_str());
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

#[test]
fn index_analytics_ext_selects_actual_session_and_sorts() {
    let low_weight = IndexAnalytics::try_from(IndexAnalyticsRow(
        "IMOEX".to_string(),
        d("2026-03-06"),
        "GAZP".to_string(),
        "ГАЗПРОМ".to_string(),
        "GAZP".to_string(),
        9.8,
        3,
        d("2026-03-06"),
    ))
    .unwrap();
    let high_weight = IndexAnalytics::try_from(IndexAnalyticsRow(
        "IMOEX".to_string(),
        d("2026-03-06"),
        "SBER".to_string(),
        "Сбербанк".to_string(),
        "SBER".to_string(),
        14.56,
        3,
        d("2026-03-06"),
    ))
    .unwrap();
    let older = IndexAnalytics::try_from(IndexAnalyticsRow(
        "IMOEX".to_string(),
        d("2026-03-05"),
        "LKOH".to_string(),
        "ЛУКОЙЛ".to_string(),
        "LKOH".to_string(),
        20.0,
        2,
        d("2026-03-05"),
    ))
    .unwrap();

    let selected = vec![low_weight, older, high_weight]
        .into_actual_by_session()
        .into_sorted_by_weight_desc();

    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].secid().as_str(), "SBER");
    assert_eq!(selected[1].secid().as_str(), "GAZP");
}

#[test]
fn parses_engine_row() {
    let row: EngineRow = serde_json::from_str(r#"[1,"stock","Фондовый рынок и рынок депозитов"]"#)
        .expect("valid engine row");
    let engine = Engine::try_from(row).expect("valid engine");
    assert_eq!(engine.id().get(), 1);
    assert_eq!(engine.name().as_str(), "stock");
}

#[test]
fn rejects_non_positive_engine_id() {
    let err = Engine::try_new(
        0,
        "stock".to_string(),
        "Фондовый рынок и рынок депозитов".to_string(),
    )
    .expect_err("must reject zero engine id");
    assert!(matches!(err, ParseEngineError::NonPositiveId(0)));
}

#[test]
fn rejects_empty_engine_name() {
    let err = Engine::try_new(1, "   ".to_string(), "Фондовый рынок".to_string())
        .expect_err("must reject empty engine name");
    assert!(matches!(
        err,
        ParseEngineError::InvalidName(ParseEngineNameError::Empty)
    ));
}

#[test]
fn rejects_engine_name_with_slash() {
    let err = Engine::try_new(
        1,
        "sto/ck".to_string(),
        "Фондовый рынок и рынок депозитов".to_string(),
    )
    .expect_err("must reject name with slash");
    assert!(matches!(
        err,
        ParseEngineError::InvalidName(ParseEngineNameError::ContainsSlash)
    ));
}

#[test]
fn parses_market_row() {
    let row: MarketRow =
        serde_json::from_str(r#"[5,"index","Индексы фондового рынка"]"#).expect("valid market row");
    let market = Market::try_from(row).expect("valid market");
    assert_eq!(market.id().get(), 5);
    assert_eq!(market.name().as_str(), "index");
}

#[test]
fn rejects_non_positive_market_id() {
    let err = Market::try_new(0, "index".to_string(), "Индексы".to_string())
        .expect_err("must reject zero market id");
    assert!(matches!(err, ParseMarketError::NonPositiveId(0)));
}

#[test]
fn rejects_empty_market_name() {
    let err = Market::try_new(5, "   ".to_string(), "Индексы".to_string())
        .expect_err("must reject empty market name");
    assert!(matches!(
        err,
        ParseMarketError::InvalidName(ParseMarketNameError::Empty)
    ));
}

#[test]
fn parses_board_row() {
    let row: BoardRow =
        serde_json::from_str(r#"[95,30,"EQCC","ЦК - режим основных торгов - безадрес.",1]"#)
            .expect("valid board row");
    let board = Board::try_from(row).expect("valid board");
    assert_eq!(board.id(), 95);
    assert_eq!(board.board_group_id(), 30);
    assert_eq!(board.boardid().as_str(), "EQCC");
    assert!(board.is_traded());
}

#[test]
fn rejects_invalid_is_traded() {
    let err = Board::try_new(
        95,
        30,
        "EQCC".to_string(),
        "ЦК - режим основных торгов - безадрес.".to_string(),
        2,
    )
    .expect_err("must reject invalid is_traded value");
    assert!(matches!(err, ParseBoardError::InvalidIsTraded(2)));
}

#[test]
fn rejects_empty_boardid() {
    let err = Board::try_new(95, 30, "   ".to_string(), "Title".to_string(), 0)
        .expect_err("must reject empty boardid");
    assert!(matches!(
        err,
        ParseBoardError::InvalidBoardId(ParseBoardIdError::Empty)
    ));
}

#[test]
fn rejects_negative_board_group_id() {
    let err = Board::try_new(95, -1, "EQCC".to_string(), "Title".to_string(), 0)
        .expect_err("must reject negative board_group_id");
    assert!(matches!(err, ParseBoardError::NegativeBoardGroupId(-1)));
}

#[test]
fn parses_security_row() {
    let row: SecurityRow = serde_json::from_str(r#"["ABIO","iАРТГЕН ао","ПАО \"Артген\"","A"]"#)
        .expect("valid security row");
    let security = Security::try_from(row).expect("valid security");
    assert_eq!(security.secid().as_str(), "ABIO");
    assert_eq!(security.shortname(), "iАРТГЕН ао");
    assert_eq!(security.status(), "A");
}

#[test]
fn rejects_empty_security_status() {
    let err = Security::try_new(
        "ABIO".to_string(),
        "iАРТГЕН ао".to_string(),
        "ПАО \"Артген\"".to_string(),
        "  ".to_string(),
    )
    .expect_err("must reject empty status");
    assert!(matches!(err, ParseSecurityError::EmptyStatus));
}

#[test]
fn rejects_security_secid_with_slash() {
    let err = Security::try_new(
        "AB/IO".to_string(),
        "iАРТГЕН ао".to_string(),
        "ПАО \"Артген\"".to_string(),
        "A".to_string(),
    )
    .expect_err("must reject secid with slash");
    assert!(matches!(
        err,
        ParseSecurityError::InvalidSecId(ParseSecIdError::ContainsSlash)
    ));
}

#[test]
fn parses_security_board() {
    let board = SecurityBoard::try_new(
        "stock".to_string(),
        "shares".to_string(),
        "TQBR".to_string(),
        1,
    )
    .expect("valid security board");
    assert_eq!(board.engine().as_str(), "stock");
    assert_eq!(board.market().as_str(), "shares");
    assert_eq!(board.boardid().as_str(), "TQBR");
    assert!(board.is_primary());
}

#[test]
fn rejects_invalid_security_board_is_primary() {
    let err = SecurityBoard::try_new(
        "stock".to_string(),
        "shares".to_string(),
        "TQBR".to_string(),
        2,
    )
    .expect_err("must reject invalid is_primary");
    assert!(matches!(err, ParseSecurityBoardError::InvalidIsPrimary(2)));
}

#[test]
fn security_boards_ext_prefers_primary_stock_board() {
    let board = vec![
        SecurityBoard::try_new(
            "currency".to_string(),
            "selt".to_string(),
            "CETS".to_string(),
            0,
        )
        .unwrap(),
        SecurityBoard::try_new(
            "stock".to_string(),
            "shares".to_string(),
            "TQTF".to_string(),
            0,
        )
        .unwrap(),
        SecurityBoard::try_new(
            "stock".to_string(),
            "shares".to_string(),
            "TQBR".to_string(),
            1,
        )
        .unwrap(),
    ]
    .into_stock_primary_or_first()
    .expect("must find stock board");

    assert_eq!(board.boardid().as_str(), "TQBR");
}

#[test]
fn parses_security_snapshot() {
    let snapshot = SecuritySnapshot::try_new("SBER".to_string(), Some(10), Some(314.8))
        .expect("valid snapshot");
    assert_eq!(snapshot.secid().as_str(), "SBER");
    assert_eq!(snapshot.lot_size(), Some(10));
    assert_eq!(snapshot.last(), Some(314.8));
}

#[test]
fn parses_candle_border_row() {
    let row: CandleBorderRow =
        serde_json::from_str(r#"["2011-12-15 10:00:00","2026-03-06 23:49:00",1,57]"#)
            .expect("valid candle border row");
    let border = CandleBorder::try_from(row).expect("valid candle border");
    assert_eq!(border.begin(), dt("2011-12-15 10:00:00"));
    assert_eq!(border.end(), dt("2026-03-06 23:49:00"));
    assert_eq!(border.interval(), CandleInterval::Minute1);
    assert_eq!(border.board_group_id(), 57);
}

#[test]
fn rejects_invalid_candle_border_interval() {
    let err = CandleBorder::try_new(
        dt("2011-12-15 10:00:00"),
        dt("2026-03-06 23:49:00"),
        999,
        57,
    )
    .expect_err("must reject unknown interval");
    assert!(matches!(
        err,
        ParseCandleBorderError::InvalidInterval(ParseCandleIntervalError::InvalidCode(999))
    ));
}

#[test]
fn rejects_negative_security_snapshot_lot_size() {
    let err = SecuritySnapshot::try_new("SBER".to_string(), Some(-1), Some(314.8))
        .expect_err("must reject negative lot size");
    assert!(matches!(
        err,
        ParseSecuritySnapshotError::NegativeLotSize(-1)
    ));
}

#[test]
fn rejects_non_finite_security_snapshot_last() {
    let err = SecuritySnapshot::try_new("SBER".to_string(), Some(10), Some(f64::INFINITY))
        .expect_err("must reject non-finite last");
    assert!(matches!(
        err,
        ParseSecuritySnapshotError::NonFiniteLast(value) if value.is_infinite()
    ));
}

#[test]
fn parses_candle_row() {
    let row: CandleRow = serde_json::from_str(
            r#"["2007-07-20 00:00:00","2007-07-20 23:59:59",109,109.2,111.11,108.01,2211623351.09,20252489]"#,
        )
        .expect("valid candle row");
    let candle = Candle::try_from(row).expect("valid candle");
    assert_eq!(candle.begin(), dt("2007-07-20 00:00:00"));
    assert_eq!(candle.end(), dt("2007-07-20 23:59:59"));
    assert_eq!(candle.close(), Some(109.2));
    assert_eq!(candle.volume(), Some(20_252_489));
}

#[test]
fn rejects_candle_with_invalid_range() {
    let err = Candle::try_new(
        dt("2007-07-20 23:59:59"),
        dt("2007-07-20 00:00:00"),
        CandleOhlcv::new(
            Some(109.0),
            Some(109.2),
            Some(111.11),
            Some(108.01),
            Some(2211623351.09),
            Some(20_252_489),
        ),
    )
    .expect_err("must reject invalid candle range");
    assert!(matches!(err, ParseCandleError::InvalidDateRange { .. }));
}

#[test]
fn rejects_candle_with_negative_volume() {
    let err = Candle::try_new(
        dt("2007-07-20 00:00:00"),
        dt("2007-07-20 23:59:59"),
        CandleOhlcv::new(
            Some(109.0),
            Some(109.2),
            Some(111.11),
            Some(108.01),
            Some(2211623351.09),
            Some(-1),
        ),
    )
    .expect_err("must reject negative volume");
    assert!(matches!(err, ParseCandleError::NegativeVolume(-1)));
}

#[test]
fn parses_trade_row() {
    let row: TradeRow =
        serde_json::from_str(r#"[15780983820,"06:59:49",314.8,1,314.8]"#).expect("valid trade row");
    let trade = Trade::try_from(row).expect("valid trade");
    assert_eq!(trade.tradeno(), 15_780_983_820);
    assert_eq!(trade.tradetime(), t("06:59:49"));
    assert_eq!(trade.price(), Some(314.8));
    assert_eq!(trade.quantity(), Some(1));
}

#[test]
fn rejects_trade_with_negative_quantity() {
    let err = Trade::try_new(
        15_780_983_820,
        t("06:59:49"),
        Some(314.8),
        Some(-1),
        Some(314.8),
    )
    .expect_err("must reject negative quantity");
    assert!(matches!(err, ParseTradeError::NegativeQuantity(-1)));
}

#[test]
fn rejects_non_positive_trade_number() {
    let err = Trade::try_new(0, t("06:59:49"), Some(314.8), Some(1), Some(314.8))
        .expect_err("must reject non-positive trade number");
    assert!(matches!(err, ParseTradeError::NonPositiveTradeNo(0)));
}

#[test]
fn candle_query_rejects_invalid_range() {
    let err = CandleQuery::try_new(
        Some(dt("2026-03-06 00:00:00")),
        Some(dt("2026-01-01 00:00:00")),
        None,
    )
    .expect_err("must reject invalid query range");
    assert!(matches!(
        err,
        ParseCandleQueryError::InvalidDateRange { .. }
    ));
}

#[test]
fn candle_query_builder_keeps_range_valid() {
    let query = CandleQuery::default()
        .with_interval(CandleInterval::Day1)
        .with_from(dt("2026-01-01 00:00:00"))
        .expect("valid from")
        .with_till(dt("2026-03-06 23:59:59"))
        .expect("valid till");

    assert_eq!(query.from(), Some(dt("2026-01-01 00:00:00")));
    assert_eq!(query.till(), Some(dt("2026-03-06 23:59:59")));
    assert_eq!(query.interval(), Some(CandleInterval::Day1));
}

#[test]
fn parses_orderbook_row() {
    let row: OrderbookLevelRow =
        serde_json::from_str(r#"["B",314.8,1200]"#).expect("valid orderbook row");
    let level = OrderbookLevel::try_from(row).expect("valid orderbook level");
    assert_eq!(level.buy_sell(), BuySell::Buy);
    assert_eq!(level.price(), 314.8);
    assert_eq!(level.quantity(), 1200);
}

#[test]
fn rejects_orderbook_row_with_invalid_side() {
    let err = OrderbookLevel::try_new("X".to_string(), Some(314.8), Some(1))
        .expect_err("must reject invalid side");
    assert!(matches!(err, ParseOrderbookError::InvalidSide(_)));
}

#[test]
fn rejects_orderbook_row_with_missing_price() {
    let err = OrderbookLevel::try_new("B".to_string(), None, Some(1))
        .expect_err("must reject missing price");
    assert!(matches!(err, ParseOrderbookError::MissingPrice));
}

#[test]
fn rejects_orderbook_row_with_negative_quantity() {
    let err = OrderbookLevel::try_new("S".to_string(), Some(314.8), Some(-1))
        .expect_err("must reject negative quantity");
    assert!(matches!(err, ParseOrderbookError::NegativeQuantity(-1)));
}

#[test]
fn pagination_builder_sets_start_and_limit() {
    let pagination = Pagination::default()
        .with_start(150)
        .with_limit(NonZeroU32::new(5_000).expect("non-zero limit"));

    assert_eq!(pagination.start, Some(150));
    assert_eq!(
        pagination.limit,
        Some(NonZeroU32::new(5_000).expect("non-zero limit"))
    );
}

#[test]
fn identifiers_support_parse_and_try_from_str() {
    let engine: EngineName = "stock".parse().expect("valid engine");
    let market = MarketName::try_from("shares").expect("valid market");
    let board = BoardId::try_from("TQBR").expect("valid board");
    let secid = SecId::try_from("SBER").expect("valid secid");
    let index: IndexId = "IMOEX".parse().expect("valid index");

    assert_eq!(engine.as_str(), "stock");
    assert_eq!(market.as_str(), "shares");
    assert_eq!(board.as_str(), "TQBR");
    assert_eq!(secid.as_str(), "SBER");
    assert_eq!(index.as_str(), "IMOEX");
}

#[test]
fn page_request_helpers_construct_expected_variants() {
    let first = PageRequest::first_page();
    assert_eq!(first, PageRequest::FirstPage);

    let pagination = Pagination::default()
        .with_start(100)
        .with_limit(NonZeroU32::new(250).expect("non-zero limit"));
    let page = PageRequest::page(pagination);
    assert_eq!(page, PageRequest::Page(pagination));

    let all = PageRequest::all(NonZeroU32::new(1_000).expect("non-zero limit"));
    assert_eq!(
        all,
        PageRequest::All {
            page_limit: NonZeroU32::new(1_000).expect("non-zero limit")
        }
    );
}

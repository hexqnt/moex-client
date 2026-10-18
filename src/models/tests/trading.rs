use super::*;

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
fn optional_counters_preserve_none_zero_and_largest_wire_integer() {
    let begin = dt("2026-03-06 10:00:00");
    for raw in [None, Some(0), Some(i64::MAX)] {
        let candle = Candle::try_new(
            begin,
            begin,
            CandleOhlcv::new(None, None, None, None, None, raw),
        )
        .expect("valid candle volume");
        let trade =
            Trade::try_new(1, t("10:00:00"), None, raw, None).expect("valid trade quantity");
        assert_eq!(candle.volume(), raw.map(|value| value as u64));
        assert_eq!(trade.quantity(), candle.volume());
    }
    assert!(matches!(
        Trade::try_new(1, t("10:00:00"), None, Some(i64::MIN), None),
        Err(ParseTradeError::NegativeQuantity(i64::MIN))
    ));
}

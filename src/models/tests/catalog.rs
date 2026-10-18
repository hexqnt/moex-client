use super::*;

#[test]
fn catalog_preserves_field_error_priority() {
    assert_eq!(
        Engine::try_new(0, "".into(), "".into()),
        Err(ParseEngineError::NonPositiveId(0))
    );
    assert_eq!(
        Engine::try_new(1, "".into(), "".into()),
        Err(ParseEngineError::InvalidName(ParseEngineNameError::Empty))
    );
    assert_eq!(
        Market::try_new(0, "".into(), "".into()),
        Err(ParseMarketError::NonPositiveId(0))
    );
    assert_eq!(
        Market::try_new(1, "".into(), "".into()),
        Err(ParseMarketError::InvalidName(ParseMarketNameError::Empty))
    );
    for (id, group, boardid, title, expected) in [
        (0, -1, "", "", ParseBoardError::NonPositiveId(0)),
        (1, -1, "", "", ParseBoardError::NegativeBoardGroupId(-1)),
        (
            1,
            0,
            "",
            "",
            ParseBoardError::InvalidBoardId(ParseBoardIdError::Empty),
        ),
        (1, 0, "TQBR", "", ParseBoardError::EmptyTitle),
        (1, 0, "TQBR", "Title", ParseBoardError::InvalidIsTraded(2)),
    ] {
        assert_eq!(
            Board::try_new(id, group, boardid.into(), title.into(), 2),
            Err(expected)
        );
    }
    for (secid, shortname, secname, expected) in [
        (
            "",
            "",
            "",
            ParseSecurityError::InvalidSecId(ParseSecIdError::Empty),
        ),
        ("SBER", "", "", ParseSecurityError::EmptyShortname),
        ("SBER", "Short", "", ParseSecurityError::EmptySecname),
        ("SBER", "Short", "Full", ParseSecurityError::EmptyStatus),
    ] {
        assert_eq!(
            Security::try_new(secid.into(), shortname.into(), secname.into(), "".into()),
            Err(expected)
        );
    }
}

#[test]
fn catalog_preserves_u32_boundaries() {
    let max = i64::from(u32::MAX);
    assert_eq!(
        Engine::try_new(max, "stock".into(), "Title".into())
            .expect("largest engine id")
            .id()
            .get(),
        u32::MAX
    );
    assert_eq!(
        Engine::try_new(max + 1, "".into(), "".into()),
        Err(ParseEngineError::IdOutOfRange(max + 1))
    );
    for raw in [0, max] {
        assert_eq!(
            Board::try_new(1, raw, "TQBR".into(), "Title".into(), 0)
                .expect("valid board group")
                .board_group_id(),
            raw as u32
        );
        assert_eq!(
            SecuritySnapshot::try_new("SBER".into(), Some(raw), None)
                .expect("valid lot size")
                .lot_size(),
            Some(raw as u32)
        );
    }
    assert_eq!(
        Board::try_new(1, max + 1, "".into(), "".into(), 2),
        Err(ParseBoardError::BoardGroupIdOutOfRange(max + 1))
    );
    assert_eq!(
        SecuritySnapshot::try_new("SBER".into(), Some(max + 1), Some(f64::NAN)),
        Err(ParseSecuritySnapshotError::LotSizeOutOfRange(max + 1))
    );
}

#[test]
fn catalog_normalizes_unicode_whitespace_and_reuses_owned_text() {
    let title = "Title".to_owned();
    let pointer = title.as_ptr();
    let engine = Engine::try_new(1, "stock".into(), title).expect("valid engine");
    assert_eq!(engine.title().as_ptr(), pointer);
    let market =
        Market::try_new(1, "shares".into(), "\u{2003} Name \u{a0}".into()).expect("valid market");
    assert_eq!(market.title(), "Name");
    assert_eq!(
        Market::try_new(1, "shares".into(), "\u{2003} \u{a0}".into()),
        Err(ParseMarketError::EmptyTitle)
    );
    assert_eq!(
        format!("{engine:?}"),
        "Engine { id: EngineId(1), name: EngineName(\"stock\"), title: \"Title\" }"
    );
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
fn parses_security_snapshot() {
    let snapshot = SecuritySnapshot::try_new("SBER".to_string(), Some(10), Some(314.8))
        .expect("valid snapshot");
    assert_eq!(snapshot.secid().as_str(), "SBER");
    assert_eq!(snapshot.lot_size(), Some(10));
    assert_eq!(snapshot.last(), Some(314.8));
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

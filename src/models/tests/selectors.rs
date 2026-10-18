use super::*;

#[test]
fn index_analytics_ext_selects_actual_session_and_sorts() {
    let low_weight = analytics("GAZP", 9.8, "2026-03-06", 3);
    let high_weight = analytics("SBER", 14.56, "2026-03-06", 3);
    let older = analytics("LKOH", 20.0, "2026-03-05", 2);

    let selected = vec![low_weight, older, high_weight]
        .into_actual_by_session()
        .into_sorted_by_weight_desc();

    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].secid().as_str(), "SBER");
    assert_eq!(selected[1].secid().as_str(), "GAZP");
}

#[test]
fn actual_index_selectors_agree_for_missing_and_present_till() {
    for (tills, expected) in [
        (vec![None, None], vec!["0", "1"]),
        (vec![None, Some(d("2026-03-06")), None], vec!["1"]),
        (
            vec![
                Some(d("2026-03-05")),
                Some(d("2026-03-06")),
                Some(d("2026-03-06")),
            ],
            vec!["1", "2"],
        ),
    ] {
        let indexes: Vec<_> = tills
            .into_iter()
            .enumerate()
            .map(|(id, till)| {
                Index::try_new(id.to_string(), "Index".to_owned(), None, till).expect("valid index")
            })
            .collect();
        assert_eq!(
            actual_indexes(&indexes)
                .map(|index| index.id().as_str())
                .collect::<Vec<_>>(),
            expected
        );
        let indexes = indexes.into_actual_by_till();
        assert_eq!(
            indexes
                .iter()
                .map(|index| index.id().as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

fn analytics(secid: &str, weight: f64, date: &str, session: i64) -> IndexAnalytics {
    IndexAnalytics::try_from(IndexAnalyticsRow(
        "IMOEX".into(),
        d(date),
        secid.into(),
        secid.into(),
        secid.into(),
        weight,
        session,
        d(date),
    ))
    .expect("valid analytics")
}

#[test]
fn actual_session_prioritizes_date_then_session_and_preserves_order() {
    let items = vec![
        analytics("OLD", 1.0, "2026-03-05", 3),
        analytics("FIRST", 1.0, "2026-03-06", 2),
        analytics("EARLY", 1.0, "2026-03-06", 1),
        analytics("SECOND", 1.0, "2026-03-06", 2),
    ]
    .into_actual_by_session();
    let ids: Vec<_> = items.iter().map(|item| item.secid().as_str()).collect();
    assert_eq!(ids, ["FIRST", "SECOND"]);
}

#[test]
fn analytics_sort_breaks_equal_weight_ties_by_secid() {
    let items = vec![
        analytics("SBER", 0.0, "2026-03-06", 1),
        analytics("VTBR", 10.0, "2026-03-06", 1),
        analytics("GAZP", 0.0, "2026-03-06", 1),
    ]
    .into_sorted_by_weight_desc();
    let ids: Vec<_> = items.iter().map(|item| item.secid().as_str()).collect();
    assert_eq!(ids, ["VTBR", "GAZP", "SBER"]);
}

#[test]
fn collection_selectors_accept_empty_collections() {
    assert!(Vec::<Index>::new().into_actual_by_till().is_empty());
    assert!(
        Vec::<IndexAnalytics>::new()
            .into_actual_by_session()
            .into_sorted_by_weight_desc()
            .is_empty()
    );
    let boards = Vec::<SecurityBoard>::new();
    assert!(boards.stock_primary_or_first().is_none());
    assert!(boards.into_stock_primary_or_first().is_none());
}

#[test]
fn board_selectors_agree_on_primary_fallback_and_missing_stock() {
    for (entries, expected) in [
        (vec![("currency", "CETS", 1)], None),
        (
            vec![
                ("currency", "CETS", 1),
                ("stock", "TQTF", 0),
                ("stock", "TQBR", 0),
            ],
            Some("TQTF"),
        ),
        (
            vec![
                ("stock", "TQTF", 0),
                ("stock", "TQBR", 1),
                ("stock", "TQTD", 1),
            ],
            Some("TQBR"),
        ),
    ] {
        let boards: Vec<_> = entries
            .into_iter()
            .map(|(engine, board, primary)| {
                SecurityBoard::try_new(engine.into(), "shares".into(), board.into(), primary)
                    .expect("valid board")
            })
            .collect();
        assert_eq!(
            boards
                .stock_primary_or_first()
                .map(|board| board.boardid().as_str()),
            expected
        );
        let selected = boards.into_stock_primary_or_first();
        assert_eq!(
            selected.as_ref().map(|board| board.boardid().as_str()),
            expected
        );
    }
}

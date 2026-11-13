use chrono::format::StrftimeItems;

use super::*;

#[test]
fn static_temporal_items_match_legacy_strftime_formats() {
    for (items, format) in [
        (DATE_ITEMS, "%Y-%m-%d"),
        (TIME_ITEMS, "%H:%M:%S"),
        (DATETIME_ITEMS, "%Y-%m-%d %H:%M:%S"),
    ] {
        assert!(
            items
                .iter()
                .eq(StrftimeItems::new(format).collect::<Vec<_>>().iter())
        );
    }
}

#[test]
fn temporal_parser_preserves_chrono_values_and_errors_at_boundaries() {
    for date in [
        "2024-02-29",
        "2023-02-29",
        "0000-01-01",
        "-0001-12-31",
        "+10000-01-01",
        "+262142-12-31",
        "-262143-01-01",
        "2026-3-6",
        "2026-03-06 ",
        "2026/03/06",
        "",
        "invalid",
    ] {
        assert_eq!(
            parse_temporal(date, DATE_ITEMS, Parsed::to_naive_date),
            NaiveDate::parse_from_str(date, "%Y-%m-%d"),
            "{date:?}"
        );
        for time in [
            "00:00:00",
            "23:59:60",
            "23:59:61",
            "24:00:00",
            "1:2:3",
            "12:00:00.5",
            "",
            "invalid",
        ] {
            let datetime = format!("{date} {time}");
            assert_eq!(
                parse_temporal(&datetime, DATETIME_ITEMS, |parsed| parsed
                    .to_naive_datetime_with_offset(0)),
                NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S"),
                "{datetime:?}"
            );
            assert_eq!(
                parse_temporal(time, TIME_ITEMS, Parsed::to_naive_time),
                NaiveTime::parse_from_str(time, "%H:%M:%S"),
                "{time:?}"
            );
        }
    }
}

use reqwest::StatusCode;

use super::*;

#[test]
fn table_request_preserves_query_order_for_multiple_tables_and_pagination() {
    let base = Url::parse("https://example.test/custom/iss/").expect("valid URL");
    let mut url =
        table_url(&base, true, "securities.json", TableSelection::Snapshots).expect("table URL");
    append_pagination_to_url(
        &mut url,
        Pagination::default()
            .with_start(0)
            .with_limit(NonZeroU32::new(2).expect("non-zero limit")),
    );
    assert_eq!(url.path(), "/custom/iss/securities.json");
    assert_eq!(
        url.query_pairs().collect::<Vec<_>>(),
        [
            ("iss.meta".into(), "on".into()),
            ("iss.only".into(), "securities,marketdata".into()),
            ("securities.columns".into(), "SECID,LOTSIZE".into()),
            ("marketdata.columns".into(), "SECID,LAST".into()),
            ("start".into(), "0".into()),
            ("limit".into(), "2".into()),
        ]
    );
}

#[test]
fn raw_request_preserves_explicit_metadata_and_repeated_parameters() {
    let base = Url::parse("https://example.test/iss/").expect("valid URL");
    let query = [
        ("iss.only".into(), "securities".into()),
        ("iss.meta".into(), "on".into()),
        ("iss.meta".into(), "off".into()),
    ];
    let (endpoint, url) =
        build_raw_request(&base, false, Some(" /iss/securities "), &query).expect("raw URL");
    assert_eq!(endpoint.as_ref(), "securities.json");
    assert_eq!(
        url.query(),
        Some("iss.only=securities&iss.meta=on&iss.meta=off")
    );
    let (_, url) = build_raw_request(&base, true, Some("engines"), &[]).expect("default metadata");
    assert_eq!(url.query(), Some("iss.meta=on"));
}

#[test]
fn payload_http_error_takes_precedence_over_content_detection() {
    let body = "я".repeat(NON_JSON_BODY_PREFIX_CHARS + 1);
    let error = validate_payload(
        "engines.json",
        StatusCode::BAD_GATEWAY,
        Some(reqwest::header::HeaderValue::from_static("text/html")),
        body,
    )
    .expect_err("HTTP error");
    assert!(
        matches!(error, MoexError::HttpStatus { endpoint, status: StatusCode::BAD_GATEWAY, content_type: Some(content_type), body_prefix } if endpoint.as_ref() == "engines.json" && content_type.as_ref() == "text/html" && body_prefix.chars().count() == NON_JSON_BODY_PREFIX_CHARS)
    );
}

#[test]
fn payload_detection_keeps_header_and_body_fallback_contracts() {
    assert_eq!(
        validate_payload(
            "test.json",
            StatusCode::OK,
            Some(reqwest::header::HeaderValue::from_static(
                "APPLICATION/JSON"
            )),
            "text".into()
        )
        .expect("JSON content type"),
        "text"
    );
    assert_eq!(
        validate_payload("test.json", StatusCode::OK, None, " \n[]".into()).expect("JSON body"),
        " \n[]"
    );
    assert!(
        matches!(validate_payload("test.json", StatusCode::OK, None, "<html>".into()), Err(MoexError::NonJsonPayload { endpoint, content_type: None, body_prefix }) if endpoint.as_ref() == "test.json" && body_prefix.as_ref() == "<html>")
    );
}

#[test]
fn payload_detection_ignores_non_text_content_type() {
    let content_type =
        reqwest::header::HeaderValue::from_bytes(b"application/\xff").expect("valid opaque header");
    assert_eq!(
        validate_payload(
            "test.json",
            StatusCode::OK,
            Some(content_type.clone()),
            "[]".into()
        )
        .expect("JSON body fallback"),
        "[]"
    );
    assert!(matches!(
        validate_payload(
            "test.json",
            StatusCode::OK,
            Some(content_type.clone()),
            "text".into()
        ),
        Err(MoexError::NonJsonPayload {
            content_type: None,
            ..
        })
    ));
    assert!(matches!(
        validate_payload(
            "test.json",
            StatusCode::BAD_GATEWAY,
            Some(content_type),
            "text".into()
        ),
        Err(MoexError::HttpStatus {
            content_type: None,
            ..
        })
    ));
}

macro_rules! raw_options_test {
    ($name:ident, $client:ty) => {
        #[test]
        fn $name() {
            let client = <$client>::new().expect("client");
            let options = IssRequestOptions::new()
                .metadata(IssToggle::On)
                .data(IssToggle::Off)
                .version(IssToggle::On)
                .json("extended");
            let json_ptr = options.json_value().unwrap().as_ptr();
            let request = client
                .raw_endpoint(IssEndpoint::Engines)
                .options(options)
                .metadata(IssToggle::Off)
                .json("compact");
            assert_eq!(request.query[4].1.as_ptr(), json_ptr);
            let (_, url) = request.build_request().expect("raw URL");
            assert_eq!(url.query(), Some("iss.only=engines&iss.meta=on&iss.data=off&iss.version=on&iss.json=extended&iss.meta=off&iss.json=compact"));
        }
    };
}

#[cfg(feature = "blocking")]
raw_options_test!(
    blocking_raw_options_transfer_json_without_copying,
    BlockingMoexClient
);
#[cfg(feature = "async")]
raw_options_test!(
    async_raw_options_transfer_json_without_copying,
    AsyncMoexClient
);

#[test]
fn table_selections_preserve_all_wire_parameters() {
    use TableSelection::*;

    let cases: &[(TableSelection, &[(&str, &str)])] = &[
        (
            Indices,
            &[
                ("iss.only", "indices"),
                ("indices.columns", "indexid,shortname,from,till"),
            ],
        ),
        (
            Analytics,
            &[
                ("iss.only", "analytics"),
                (
                    "analytics.columns",
                    "indexid,tradedate,ticker,shortnames,secids,weight,tradingsession,trade_session_date",
                ),
            ],
        ),
        (
            Turnovers,
            &[
                ("iss.only", "turnovers"),
                (
                    "turnovers.columns",
                    "NAME,ID,VALTODAY,VALTODAY_USD,NUMTRADES,UPDATETIME,TITLE",
                ),
            ],
        ),
        (
            Engines,
            &[
                ("iss.only", "engines"),
                ("engines.columns", "id,name,title"),
            ],
        ),
        (
            Markets,
            &[
                ("iss.only", "markets"),
                ("markets.columns", "id,NAME,title"),
            ],
        ),
        (
            Boards,
            &[
                ("iss.only", "boards"),
                (
                    "boards.columns",
                    "id,board_group_id,boardid,title,is_traded",
                ),
            ],
        ),
        (
            SecurityBoards,
            &[
                ("iss.only", "boards"),
                ("boards.columns", "engine,market,boardid,is_primary"),
            ],
        ),
        (
            Securities,
            &[
                ("iss.only", "securities"),
                ("securities.columns", "SECID,SHORTNAME,SECNAME,STATUS"),
            ],
        ),
        (
            Snapshots,
            &[
                ("iss.only", "securities,marketdata"),
                ("securities.columns", "SECID,LOTSIZE"),
                ("marketdata.columns", "SECID,LAST"),
            ],
        ),
        (
            Orderbook,
            &[
                ("iss.only", "orderbook"),
                ("orderbook.columns", "BUYSELL,PRICE,QUANTITY"),
            ],
        ),
        (
            Candles,
            &[
                ("iss.only", "candles"),
                (
                    "candles.columns",
                    "begin,end,open,close,high,low,value,volume",
                ),
            ],
        ),
        (
            Trades,
            &[
                ("iss.only", "trades"),
                ("trades.columns", "TRADENO,TRADETIME,PRICE,QUANTITY,VALUE"),
            ],
        ),
        (
            SecStats,
            &[
                ("iss.only", "secstats"),
                (
                    "secstats.columns",
                    "SECID,BOARDID,VOLTODAY,VALTODAY,HIGHBID,LOWOFFER,LASTOFFER,LASTBID,OPEN,LOW,HIGH,LAST,NUMTRADES,WAPRICE",
                ),
            ],
        ),
        (MetadataOnly, &[]),
        #[cfg(feature = "news")]
        (
            SiteNews,
            &[
                ("iss.only", "sitenews"),
                ("sitenews.columns", "id,tag,title,published_at,modified_at"),
            ],
        ),
        #[cfg(feature = "news")]
        (
            Events,
            &[
                ("iss.only", "events"),
                ("events.columns", "id,tag,title,from,modified_at"),
            ],
        ),
        #[cfg(feature = "history")]
        (HistoryDates, &[("iss.only", "dates")]),
        #[cfg(feature = "history")]
        (
            History,
            &[
                ("iss.only", "history"),
                (
                    "history.columns",
                    "BOARDID,TRADEDATE,SECID,NUMTRADES,VALUE,OPEN,LOW,HIGH,CLOSE,VOLUME,DURATION,YIELD",
                ),
            ],
        ),
    ];
    for &(selection, expected) in cases {
        let url = table_url(
            &Url::parse("https://example.test/iss/").unwrap(),
            false,
            "demo.json",
            selection,
        )
        .unwrap();
        let mut query = url.query_pairs();
        assert_eq!(query.next(), Some(("iss.meta".into(), "off".into())));
        assert!(
            query.eq(expected
                .iter()
                .map(|&(key, value)| (key.into(), value.into())))
        );
    }
}

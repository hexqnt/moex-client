#![cfg(any(feature = "blocking", feature = "async"))]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::num::NonZeroU32;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use moex_client::{IssEndpoint, IssRequestOptions, IssToggle, MoexError};
use reqwest::{StatusCode, Url};

fn server(responses: Vec<(u16, &'static str, &'static str)>) -> (Url, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = Url::parse(&format!("http://{}/iss/", listener.local_addr().unwrap())).unwrap();
    let (sender, requests) = mpsc::channel();
    std::thread::spawn(move || {
        for (status, content_type, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(&stream);
            let mut request = String::new();
            reader.read_line(&mut request).unwrap();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
            }
            write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nX-ISS-Test: preserved\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            let _ = sender.send(request);
        }
    });
    (base_url, requests)
}

fn query(requests: &Receiver<String>) -> (String, Vec<(String, String)>) {
    let request = requests.recv_timeout(Duration::from_secs(3)).unwrap();
    let target = request.split_whitespace().nth(1).unwrap();
    let url = Url::parse(&format!("http://localhost{target}")).unwrap();
    (url.path().into(), url.query_pairs().into_owned().collect())
}

macro_rules! call {
    (blocking, $operation:expr) => {
        $operation
    };
    (asynchronous, $operation:expr) => {
        $operation.await
    };
}

macro_rules! transport_test {
    (blocking, fn $name:ident() $body:block) => {
        #[test]
        fn $name() $body
    };
    (asynchronous, fn $name:ident() $body:block) => {
        #[test]
        fn $name() {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async $body);
        }
    };
}

macro_rules! transport_tests {
    ($module:ident, $client:ty, $mode:ident) => {
        mod $module {
            use super::*;

            fn server(responses: Vec<(u16, &'static str, &'static str)>) -> ($client, Receiver<String>) {
                let (base_url, requests) = super::server(responses);
                let client = <$client>::builder()
                    .base_url(base_url)
                    .no_proxy()
                    .timeout(Duration::from_secs(3))
                    .build()
                    .unwrap();
                (client, requests)
            }

            transport_test!($mode, fn typed_request_preserves_path_query_order_and_decodes_response() {
                let (client, requests) = server(vec![(
                    200,
                    "application/json",
                    r#"{"engines":{"data":[[1,"stock","Stock market"]]}}"#,
                )]);
                assert_eq!(call!($mode, client.engines()).unwrap()[0].name().as_str(), "stock");
                assert_eq!(
                    query(&requests),
                    (
                        "/iss/engines.json".into(),
                        vec![
                            ("iss.meta".into(), "off".into()),
                            ("iss.only".into(), "engines".into()),
                            ("engines.columns".into(), "id,name,title".into()),
                        ]
                    )
                );
            });

            transport_test!($mode, fn raw_request_preserves_explicit_metadata_duplicates_and_encoding() {
                let (client, requests) = server(vec![(200, "application/json", "{}")]);
                call!($mode, client
                    .raw_endpoint(IssEndpoint::Engines)
                    .options(
                        IssRequestOptions::new()
                            .metadata(IssToggle::On)
                            .data(IssToggle::Off),
                    )
                    .param("q", "A&B +/Я")
                    .param("q", "second")
                    .send_payload())
                    .unwrap();
                assert_eq!(
                    query(&requests),
                    (
                        "/iss/engines.json".into(),
                        vec![
                            ("iss.only".into(), "engines".into()),
                            ("iss.meta".into(), "on".into()),
                            ("iss.data".into(), "off".into()),
                            ("q".into(), "A&B +/Я".into()),
                            ("q".into(), "second".into()),
                        ]
                    )
                );
            });

            transport_test!($mode, fn raw_response_preserves_error_status_headers_and_body() {
                let (client, _) = server(vec![(503, "text/html", "<h1>Unavailable</h1>")]);
                let response = call!($mode, client.raw().path("engines").send_response()).unwrap();
                assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
                assert_eq!(response.headers()["x-iss-test"], "preserved");
                assert_eq!(response.body(), "<h1>Unavailable</h1>");
            });

            transport_test!($mode, fn payload_validation_preserves_error_context_and_json_heuristic() {
                let (client, _) = server(vec![
                    (429, "text/html", "Too many requests"),
                    (200, "text/html", "<html>Not JSON</html>"),
                    (200, "text/plain", " {\"ok\":true}"),
                ]);
                let error = call!($mode, client
                    .raw()
                    .path("/iss/engines")
                    .send_payload())
                    .unwrap_err();
                assert!(
                    matches!(&error, MoexError::HttpStatus { endpoint, .. } if &**endpoint == "engines.json")
                );
                assert!(error.is_retryable());
                assert_eq!(error.response_body_prefix(), Some("Too many requests"));
                let error = call!($mode, client.raw().path("engines").send_payload()).unwrap_err();
                assert!(
                    matches!(&error, MoexError::NonJsonPayload { endpoint, .. } if &**endpoint == "engines.json")
                );
                assert_eq!(error.response_body_prefix(), Some("<html>Not JSON</html>"));
                assert_eq!(
                    call!($mode, client.raw().path("engines").send_payload()).unwrap(),
                    " {\"ok\":true}"
                );
            });

            transport_test!($mode, fn pagination_retries_failed_page_and_stops_after_short_page() {
                let (client, requests) = server(vec![
                    (
                        200,
                        "application/json",
                        r#"{"securities":{"data":[["SBER","Sber","Sberbank","A"],["GAZP","Gaz","Gazprom","A"]]}}"#,
                    ),
                    (503, "text/plain", "retry"),
                    (
                        200,
                        "application/json",
                        r#"{"securities":{"data":[["ROSN","Ros","Rosneft","A"]]}}"#,
                    ),
                ]);
                let mut pages = client.global_securities_pages(NonZeroU32::new(2).unwrap());
                assert_eq!(call!($mode, pages.next_page()).unwrap().unwrap().len(), 2);
                assert!(call!($mode, pages.next_page()).unwrap_err().is_retryable());
                assert_eq!(
                    call!($mode, pages.next_page()).unwrap().unwrap()[0].secid().as_str(),
                    "ROSN"
                );
                assert!(call!($mode, pages.next_page()).unwrap().is_none());
                for expected in ["0", "2", "2"] {
                    let (_, query) = query(&requests);
                    assert_eq!(
                        &query[query.len() - 2..],
                        &[
                            ("start".into(), expected.into()),
                            ("limit".into(), "2".into())
                        ]
                    );
                }
            });

            transport_test!($mode, fn pagination_rejects_repeated_page_without_returning_partial_collection() {
                let page = r#"{"securities":{"data":[["SBER","Sber","Sberbank","A"]]}}"#;
                let (client, _) = server(vec![
                    (200, "application/json", page),
                    (200, "application/json", page),
                ]);
                let error = call!($mode, client
                    .global_securities_pages(NonZeroU32::new(1).unwrap())
                    .all())
                    .unwrap_err();
                assert!(matches!(
                    error,
                    MoexError::PaginationStuck {
                        start: 1,
                        limit: 1,
                        ..
                    }
                ));
            });

            transport_test!($mode, fn pagination_collects_full_and_short_pages_in_response_order() {
                let (client, requests) = server(vec![
                    (
                        200,
                        "application/json",
                        r#"{"securities":{"data":[["SBER","Sber","Sberbank","A"],["GAZP","Gaz","Gazprom","A"]]}}"#,
                    ),
                    (
                        200,
                        "application/json",
                        r#"{"securities":{"data":[["ROSN","Ros","Rosneft","A"]]}}"#,
                    ),
                ]);
                let securities = call!($mode, client
                    .global_securities_pages(NonZeroU32::new(2).unwrap())
                    .all())
                    .unwrap();
                assert_eq!(
                    securities
                        .iter()
                        .map(|security| security.secid().as_str())
                        .collect::<Vec<_>>(),
                    ["SBER", "GAZP", "ROSN"]
                );
                for expected_start in ["0", "2"] {
                    let (path, query) = query(&requests);
                    assert_eq!(path, "/iss/securities.json");
                    assert_eq!(
                        &query[query.len() - 2..],
                        &[
                            ("start".into(), expected_start.into()),
                            ("limit".into(), "2".into())
                        ]
                    );
                }
            });
        }
    };
}

#[cfg(feature = "blocking")]
transport_tests!(blocking, moex_client::blocking::Client, blocking);
#[cfg(feature = "async")]
transport_tests!(asynchronous, moex_client::r#async::Client, asynchronous);

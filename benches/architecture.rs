use std::hint::black_box;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::Instant;

use moex_client::blocking::Client;
use moex_client::models::{BoardId, EngineName, IndexId, MarketName, SecId};
use moex_client::{IssEndpoint, IssRequestOptions, IssToggle, decode};
use serde::Deserialize;

#[derive(Deserialize)]
struct SecurityRow {
    #[serde(rename = "SECID")]
    secid: String,
    #[serde(rename = "SHORTNAME")]
    shortname: String,
}

fn table(name: &str, columns: &str, row: &str) -> String {
    let rows = std::iter::repeat_n(row, 100).collect::<Vec<_>>().join(",");
    format!(r#"{{"{name}":{{"columns":{columns},"data":[{rows}]}}}}"#)
}

fn measure(name: &str, mut action: impl FnMut()) {
    if std::env::var("MOEX_BENCH_CASE").is_ok_and(|filter| filter != name) {
        return;
    }
    for _ in 0..100 {
        action();
    }
    let iterations = 500;
    let mut samples = [0; 9];
    for sample in &mut samples {
        let start = Instant::now();
        for _ in 0..iterations {
            action();
        }
        *sample = start.elapsed().as_nanos() / iterations;
    }
    samples.sort_unstable();
    println!(
        "{name}: {} ns/iteration (p10={}, p90={})",
        samples[samples.len() / 2],
        samples[1],
        samples[samples.len() - 2]
    );
}

fn main() {
    let indexes = table(
        "indices",
        r#"["indexid","shortname","from","till"]"#,
        r#"["IMOEX","Индекс МосБиржи","2001-01-03","2026-03-05"]"#,
    );
    let securities = table(
        "securities",
        r#"["SECID","SHORTNAME","SECNAME","STATUS"]"#,
        r#"["SBER","Сбербанк","ПАО Сбербанк","A"]"#,
    );
    let candles = table(
        "candles",
        r#"["begin","end","open","close","high","low","value","volume"]"#,
        r#"["2026-03-06 10:00:00","2026-03-06 10:59:59",314,315,316,313,30000,100]"#,
    );
    let history = table(
        "history",
        r#"["BOARDID","TRADEDATE","SECID","NUMTRADES","VALUE","OPEN","LOW","HIGH","CLOSE","VOLUME","DURATION","YIELD"]"#,
        r#"["TQBR","2026-03-06","SBER",100,30000,314,313,316,315,100,1733.5,15.49]"#,
    );
    let events = table(
        "events",
        r#"["id","tag","title","from","modified_at"]"#,
        r#"[77,"site","Технические работы",null,"2026-03-06 22:00:00"]"#,
    );
    let analytics = table(
        "analytics",
        "[]",
        r#"["IMOEX","2026-03-06","SBER","Сбербанк","SBER",12.5,3,"2026-03-06"]"#,
    );
    let engines = table("engines", "[]", r#"[1,"stock","Фондовый рынок"]"#);
    let trades = table("trades", "[]", r#"[15780983820,"06:59:49",314.8,1,314.8]"#);
    let turnovers = table(
        "turnovers",
        "[]",
        r#"["TOTAL",1,100,1,10,"2026-03-06 10:00:00","Total"]"#,
    );
    let secstats = table(
        "secstats",
        "[]",
        r#"["SBER","TQBR",100,30000,314,316,316,314,314,313,316,315,100,315]"#,
    );
    let snapshots = r#"{"securities":{"data":[["SBER",10],["GAZP",10]]},"marketdata":{"data":[["SBER",315.0],["ROSN",410.0]]}}"#;
    measure("indexes_100", || {
        black_box(decode::indexes_json(black_box(&indexes)).unwrap());
    });
    measure("candles_100", || {
        black_box(decode::candles_json(black_box(&candles)).unwrap());
    });
    measure("history_100", || {
        black_box(decode::history_json(black_box(&history)).unwrap());
    });
    measure("events_100", || {
        black_box(decode::events_json(black_box(&events)).unwrap());
    });
    measure("analytics_100", || {
        black_box(decode::index_analytics_json(black_box(&analytics)).unwrap());
    });
    measure("engines_100", || {
        black_box(decode::engines_json(black_box(&engines)).unwrap());
    });
    measure("trades_100", || {
        black_box(decode::trades_json(black_box(&trades)).unwrap());
    });
    measure("turnovers_100", || {
        black_box(decode::turnovers_json(black_box(&turnovers)).unwrap());
    });
    measure("secstats_100", || {
        black_box(decode::secstats_json(black_box(&secstats)).unwrap());
    });
    measure("snapshots_join", || {
        black_box(decode::board_security_snapshots_json(black_box(snapshots)).unwrap());
    });
    measure("raw_owned_100", || {
        let rows: Vec<SecurityRow> =
            decode::raw_table_rows_json(black_box(&securities), "bench", "securities").unwrap();
        black_box(
            rows.iter()
                .map(|row| row.secid.len() + row.shortname.len())
                .sum::<usize>(),
        );
    });
    measure("raw_borrowed_100", || {
        black_box(
            decode::raw_table_view_json(black_box(&securities), "bench", "securities").unwrap(),
        );
    });
    measure("raw_tables_100", || {
        let mut tables = decode::raw_tables_json(black_box(&securities), "bench").unwrap();
        black_box(tables.take_rows::<SecurityRow>("securities").unwrap());
    });
    measure("identifier_borrowed", || {
        black_box(SecId::try_from(black_box(" SBER ")).unwrap());
    });
    measure("identifier_owned", || {
        black_box(SecId::try_from(black_box(String::from("SBER"))).unwrap());
    });
    let engine = EngineName::try_from("stock").unwrap();
    let market = MarketName::try_from("shares").unwrap();
    let board = BoardId::try_from("TQBR").unwrap();
    let security = SecId::try_from("SBER").unwrap();
    measure("endpoint_path", || {
        black_box(
            IssEndpoint::Trades {
                engine: &engine,
                market: &market,
                board: &board,
                security: &security,
            }
            .path(),
        );
    });
    let http = reqwest::blocking::Client::builder()
        .no_proxy()
        .build()
        .unwrap();
    let client = Client::with_client(http).unwrap();
    measure("raw_builder", || {
        let _ = black_box(
            client
                .raw_endpoint(IssEndpoint::Indexes)
                .options(IssRequestOptions::new().metadata(IssToggle::Off))
                .columns("indices", "indexid,shortname")
                .param("start", "0"),
        );
    });
    let index = IndexId::try_from("IMOEX").unwrap();
    measure("scopes", || {
        black_box(client.index(black_box(&index)).unwrap());
        black_box(
            client
                .engine(black_box(&engine))
                .unwrap()
                .market(black_box(&market))
                .unwrap()
                .board(black_box(&board))
                .unwrap()
                .security(black_box(&security))
                .unwrap(),
        );
    });

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url =
        reqwest::Url::parse(&format!("http://{}/iss/", listener.local_addr().unwrap())).unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
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
            let body = if request.contains("/iss/securities.json?") {
                if request.contains("start=0&") {
                    r#"{"securities":{"data":[["SBER","Sber","Sberbank","A"],["GAZP","Gaz","Gazprom","A"]]}}"#
                } else {
                    r#"{"securities":{"data":[["ROSN","Ros","Rosneft","A"]]}}"#
                }
            } else {
                "{}"
            };
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
    });
    let client = Client::builder()
        .base_url(base_url)
        .no_proxy()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap();
    measure("request_http", || {
        black_box(
            client
                .raw_endpoint(IssEndpoint::Engines)
                .param("q", "A&B +/Я")
                .send_payload()
                .unwrap(),
        );
    });
    measure("pagination_http", || {
        black_box(
            client
                .global_securities_pages(const { std::num::NonZeroU32::new(2).unwrap() })
                .all()
                .unwrap(),
        );
    });
}

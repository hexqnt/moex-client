# moex-client

[![CI](https://github.com/hexqnt/moex-client/actions/workflows/ci.yml/badge.svg)](https://github.com/hexqnt/moex-client/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/moex-client.svg)](https://crates.io/crates/moex-client)
[![docs.rs](https://docs.rs/moex-client/badge.svg)](https://docs.rs/moex-client)

Неофициальная библиотека для типизированной выгрузки данных из ISS API Московской биржи.

## Принципы API

- Публичный слой работает со строгими доменными типами (`SecId`, `BoardId`, `EngineName`, `MarketName`, `IndexId`).
- Внешние данные ISS валидируются на границе и преобразуются в модели через `TryFrom`/`try_new`.
- Для пост-обработки коллекций используется fluent-стиль через extension-traits (`moex_client::prelude::*`).
- Для непокрытых endpoint-ов есть raw escape hatch с типизированными `iss.*` опциями.

## Покрытие выгрузок ISS

Источник перечня: <https://iss.moex.com/iss/reference/>

| Возможность выгрузки                                  | Эндпоинт из ISS reference                                                                              | Реализовано |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ----------- |
| Список торговых систем                                | `/iss/engines`                                                                                         | [x]         |
| Список рынков торговой системы                        | `/iss/engines/[engine]/markets`                                                                        | [x]         |
| Справочник режимов торгов                             | `/iss/engines/[engine]/markets/[market]/boards`                                                        | [x]         |
| Инструменты по режиму торгов                          | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities`                                     | [x]         |
| Стакан по инструменту и режиму                        | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/orderbook`                | [x]         |
| Сделки по инструменту и режиму                        | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/trades`                   | [x]         |
| Свечи по инструменту и режиму                         | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/candles`                  | [x]         |
| Список индексов фондового рынка                       | `/iss/statistics/engines/stock/markets/index/analytics`                                                | [x]         |
| Аналитика состава индекса                             | `/iss/statistics/engines/stock/markets/index/analytics/[indexid]`                                      | [x]         |
| Общий список бумаг MOEX                               | `/iss/securities`                                                                                      | [x]         |
| Карточка бумаги по `secid`                            | `/iss/securities/[security]`                                                                           | [x]         |
| Инструменты на уровне рынка (без `boards/[board]`)    | `/iss/engines/[engine]/markets/[market]/securities`                                                    | [x]         |
| Сделки/стаканы на уровне рынка (без `boards/[board]`) | `/iss/engines/[engine]/markets/[market]/trades`, `/orderbook`                                          | [x]         |
| Границы свечей по инструменту                         | `/iss/engines/[engine]/markets/[market]/securities/[security]/candleborders`                           | [x]         |
| История: доступные даты по инструменту                | `/iss/history/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/dates`            | [x]         |
| Исторические выгрузки                                 | `/iss/history/...`                                                                                     | [x]         |
| Обороты и secstats                                    | `/iss/turnovers`, `/iss/engines/[engine]/turnovers`, `/iss/engines/[engine]/markets/[market]/secstats` | [x]         |
| ReferenceData 2.0                                     | `/iss/referencedata/...`                                                                               | [ ]         |
| Новости и события                                     | `/iss/sitenews`, `/iss/events`                                                                         | [x]         |

## Feature flags

По умолчанию включены: `blocking`, `rustls-tls`.

- `async` — асинхронный клиент.
- `blocking` — блокирующий клиент.
- `history` — эндпоинты `/history/...`.
- `news` — эндпоинты `/sitenews` и `/events`.
- `rustls-tls` / `native-tls` — выбор TLS backend для `reqwest`.

- Только async API: `moex-client = { version = "...", default-features = false, features = ["async", "rustls-tls"] }`
- Только blocking API: `moex-client = { version = "...", default-features = false, features = ["blocking", "rustls-tls"] }`

## Минимальный пример

```rust
use moex_client::blocking::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent_from_crate()
        .metadata(false)
        .build()?;
    let engines = client.engines()?;
    println!("engines: {}", engines.len());
    Ok(())
}
```

Асинхронный вариант:

```rust
use moex_client::r#async::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent_from_crate()
        .metadata(false)
        .build()?;
    let engines = client.engines().await?;
    println!("engines: {}", engines.len());

    let indexid = moex_client::models::IndexId::try_from("IMOEX")?;
    let mut pages = client
        .index(indexid)?
        .analytics_pages(std::num::NonZeroU32::new(5000).expect("non-zero"));
    while let Some(_page) = pages.next_page().await? {
        // incremental processing
    }

    Ok(())
}
```

## Proxy

Можно добавить proxy-правила через builder:

```rust
use moex_client::blocking::Client;

let proxy = reqwest::Proxy::all("http://127.0.0.1:3128")?;
let _client = Client::builder().proxy(proxy).build()?;
```

Если нужно отключить proxy из окружения и системных настроек:

```rust
use moex_client::blocking::Client;

let _client = Client::builder().no_proxy().build()?;
```

Для асинхронного клиента API аналогичный (`moex_client::r#async::Client::builder()`).

## Retry helper

Для транспортного ретрая доступен helper без привязки к конкретным endpoint-ам:

```rust
use std::num::NonZeroU32;
use std::time::Duration;

use moex_client::blocking::Client;
use moex_client::{RetryPolicy, with_retry};

fn fetch_with_retry(client: &Client) -> Result<(), moex_client::MoexError> {
    let policy = RetryPolicy::new(NonZeroU32::new(3).expect("non-zero"))
        .with_delay(Duration::from_millis(400));

    let _engines = with_retry(policy, || client.engines())?;
    Ok(())
}
```

Асинхронный вариант использует `with_retry_async(...)` и принимает `sleep`-функцию
из runtime приложения (например, `tokio::time::sleep`).

## Rate limit helper

```rust
use std::time::Duration;

use moex_client::blocking::Client;
use moex_client::{RateLimit, RateLimiter, with_rate_limit};

fn fetch_with_rate_limit(client: &Client) -> Result<(), moex_client::MoexError> {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_millis(250)));

    let _engines = with_rate_limit(&mut limiter, || client.engines())?;
    Ok(())
}
```

Асинхронный вариант использует `with_rate_limit_async(...)` и, как `with_retry_async(...)`,
принимает функцию `sleep` из runtime приложения.

Для blocking-клиента лимит можно зафиксировать в builder-е и применять автоматически ко всем запросам:

```rust
use std::time::Duration;

use moex_client::blocking::Client;
use moex_client::RateLimit;

let _client = Client::builder()
    .rate_limit(RateLimit::every(Duration::from_millis(250)))
    .build()?;
```

Для async-клиента вместе с лимитом нужно задать функцию ожидания из вашего runtime:

```rust
use std::time::Duration;

use moex_client::r#async::Client;
use moex_client::RateLimit;

let _client = Client::builder()
    .rate_limit(RateLimit::every(Duration::from_millis(250)))
    .rate_limit_sleep(tokio::time::sleep)
    .build()?;
```

## Fluent-селекторы и prelude

Слой `Client` остаётся endpoint-level, а бизнес-выборка делается через extension-traits:
`IndexesExt`, `IndexAnalyticsExt`, `SecurityBoardsExt`.

Для подключения fluent-методов достаточно одного импорта:

```rust
use moex_client::prelude::*;
```

Пример:

```rust
use std::num::NonZeroU32;

use moex_client::blocking::Client;
use moex_client::models::{PageRequest, SecId};
use moex_client::prelude::*;

fn demo(client: &Client, indexid: &moex_client::models::IndexId, secid: &SecId) -> Result<(), moex_client::MoexError> {
    let _indexes = client.indexes()?.into_actual_by_till();

    let _components = client
        .index(indexid.clone())?
        .analytics(PageRequest::all(NonZeroU32::new(5000).expect("non-zero limit")))?
        .into_actual_by_session()
        .into_sorted_by_weight_desc();

    let _board = client
        .security(secid.clone())?
        .boards()?
        .into_stock_primary_or_first();

    let _security_info = client.security(secid.clone())?.info()?;

    Ok(())
}
```

## Raw ISS запросы (escape hatch)

Для endpoint-ов, которые ещё не покрыты строгими методами, доступен low-level builder:

```rust
use moex_client::blocking::Client;
use serde_json::Value;

fn demo_raw(client: &Client) -> Result<Value, moex_client::MoexError> {
    client
        .raw()
        .path("history/engines/stock/markets/shares/securities")
        .param("date", "2026-03-06")
        .only("history")
        .columns("history", "SECID,BOARDID,CLOSE")
        .send_json::<Value>()
}
```

Если не хочется собирать path строкой, можно использовать типизированный endpoint:

```rust
use moex_client::blocking::Client;
use moex_client::{IssEndpoint, MoexError};
use moex_client::models::{BoardId, EngineName, MarketName};

fn demo_typed_raw(client: &Client) -> Result<String, MoexError> {
    let engine = EngineName::try_from("stock")?;
    let market = MarketName::try_from("shares")?;
    let board = BoardId::try_from("TQBR")?;

    client
        .raw_endpoint(IssEndpoint::Securities {
            engine: &engine,
            market: &market,
            board: &board,
        })
        .columns("securities", "SECID,SHORTNAME")
        .send_payload()
}
```

`path(...)` принимает формы вида `engines`, `engines.json`, `/iss/engines`, `/iss/engines.json`.
Путь нормализуется к относительному `<endpoint>.json`; query-строку в `path` передавать нельзя, для этого используйте `.param(...)`.

Системные параметры `iss.*` можно задавать через типизированные опции, а при необходимости
получать полный HTTP-ответ (status/headers/body):

```rust
use moex_client::blocking::Client;
use moex_client::{IssRequestOptions, IssToggle};

fn demo_raw_response(client: &Client) -> Result<u16, moex_client::MoexError> {
    let response = client
        .raw()
        .path("engines")
        .options(
            IssRequestOptions::new()
                .metadata(IssToggle::Off)
                .data(IssToggle::On)
                .version(IssToggle::On)
                .json("extended"),
        )
        .send_response()?;

    Ok(response.status().as_u16())
}
```

Если нужен не весь JSON, а только строки конкретной таблицы, можно декодировать их сразу в свой тип:

```rust
use moex_client::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HistoryCloseRow {
    #[serde(rename = "SECID")]
    secid: String,
    #[serde(rename = "BOARDID")]
    boardid: String,
    #[serde(rename = "CLOSE")]
    close: Option<f64>,
}

fn demo_raw_table(client: &Client) -> Result<Vec<HistoryCloseRow>, moex_client::MoexError> {
    client
        .raw()
        .path("history/engines/stock/markets/shares/securities")
        .param("date", "2026-03-06")
        .only("history")
        .columns("history", "SECID,BOARDID,CLOSE")
        .send_table("history")
}
```

Для парсинга payload-ов без клиента есть отдельный модуль `moex_client::decode`:

```rust
fn parse_indexes(payload: &str) -> Result<Vec<moex_client::models::Index>, moex_client::MoexError> {
    moex_client::decode::indexes_json(payload)
}
```

Также доступен generic-декодер выбранной таблицы в пользовательский тип:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CustomRow {
    #[serde(rename = "SECID")]
    secid: String,
}

fn parse_custom_table(payload: &str) -> Result<Vec<CustomRow>, moex_client::MoexError> {
    moex_client::decode::raw_table_rows_json(payload, "custom/endpoint.json", "securities")
}
```

Если из одного payload нужно извлечь несколько таблиц, можно один раз разобрать top-level блоки:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SecurityRow {
    #[serde(rename = "SECID")]
    secid: String,
}

#[derive(Debug, Deserialize)]
struct MarketDataRow {
    #[serde(rename = "SECID")]
    secid: String,
}

fn parse_two_tables(payload: &str) -> Result<(Vec<SecurityRow>, Vec<MarketDataRow>), moex_client::MoexError> {
    let mut tables = moex_client::decode::raw_tables_json(payload, "custom/endpoint.json")?;
    let securities = tables.take_rows("securities")?;
    let marketdata = tables.take_rows("marketdata")?;
    Ok((securities, marketdata))
}
```

`take_rows(...)` забирает таблицу из внутреннего кэша `RawTables`.

Если нужен borrowed-доступ к ячейкам без `DeserializeOwned`, используйте `raw_table_view_json`:

```rust
fn parse_borrowed(payload: &str) -> Result<(), moex_client::MoexError> {
    let table = moex_client::decode::raw_table_view_json(payload, "custom/endpoint.json", "securities")?;
    let secid: &str = table
        .deserialize_value(0, "SECID")?
        .unwrap_or_default();
    assert!(!secid.is_empty());
    Ok(())
}
```

## Scoped API, PageRequest и paginator-ы

Для endpoint-ов `index_analytics/securities/trades/candles` доступен единый режим страниц через `PageRequest`
и fluent scope с фиксацией `engine/market/board/security`.

Также есть ленивые paginator-объекты с `next_page()/all()`:

```rust
use std::num::NonZeroU32;

use moex_client::blocking::Client;
use moex_client::models::{BoardId, CandleQuery, EngineName, MarketName, PageRequest, SecId};

fn demo_scoped(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let engine = EngineName::try_from("stock")?;
    let market = MarketName::try_from("shares")?;
    let board = BoardId::try_from("TQBR")?;
    let secid = SecId::try_from("SBER")?;

    let _global_securities = client.global_securities_query(PageRequest::first_page())?;

    let market_scope = client
        .engine(engine.clone())?
        .market(market.clone())?;

    let _market_securities = market_scope
        .securities(PageRequest::first_page())?;

    let _market_trades = market_scope
        .trades(PageRequest::first_page())?;

    let _turnovers = client.turnovers()?;
    let _engine_turnovers = client.engine_turnovers(&engine)?;

    let _secstats = market_scope.secstats(PageRequest::first_page())?;

    let _sitenews = client.sitenews_query(PageRequest::first_page())?;
    let _events = client.events_query(PageRequest::first_page())?;

    let _candle_borders = market_scope.candle_borders(&secid)?;

    let security_scope = market_scope
        .board(board.clone())?
        .security(secid.clone())?;

    let _history_dates = security_scope.history_dates()?;

    let _history = security_scope.history(PageRequest::first_page())?;

    let _trades = security_scope
        .trades(PageRequest::all(NonZeroU32::new(1000).expect("non-zero")))?;

    let _candles = security_scope
        .candles(CandleQuery::default(), PageRequest::first_page())?;

    let _all_trades = security_scope
        .trades_pages(NonZeroU32::new(1000).expect("non-zero"))
        .all()?;

    let mut pages = client
        .index(moex_client::models::IndexId::try_from("IMOEX")?)?
        .analytics_pages(NonZeroU32::new(5000).expect("non-zero"));
    while let Some(_page) = pages.next_page()? {
        // incremental processing
    }

    Ok(())
}
```

## Пример выгрузки актуальных индексов

```bash
cargo run --example actual_indexes_dump
```

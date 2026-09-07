# moex-client

[🇺🇸 English](./README.md) · [🇷🇺 Русский](./README.ru.md)

[![CI](https://github.com/hexqnt/moex-client/actions/workflows/ci.yml/badge.svg)](https://github.com/hexqnt/moex-client/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/moex-client.svg)](https://crates.io/crates/moex-client)
[![docs.rs](https://docs.rs/moex-client/badge.svg)](https://docs.rs/moex-client)

An unofficial, strictly typed Rust client for the Moscow Exchange ISS API.

## API design

- The public API uses strict domain types (`SecId`, `BoardId`, `EngineName`, `MarketName`, and `IndexId`).
- ISS data is validated at the boundary and converted into models through `TryFrom` and `try_new`.
- Extension traits provide fluent collection processing (`moex_client::prelude::*`).
- Endpoints without a strict API remain accessible through a raw request builder with typed `iss.*` options.

## ISS coverage

Endpoint reference: <https://iss.moex.com/iss/reference/>

| Data                                                              | ISS endpoint                                                                                           | Implemented |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ----------- |
| Trading engines                                                   | `/iss/engines`                                                                                         | [x]         |
| Markets of a trading engine                                       | `/iss/engines/[engine]/markets`                                                                        | [x]         |
| Trading boards                                                    | `/iss/engines/[engine]/markets/[market]/boards`                                                        | [x]         |
| Securities on a board                                             | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities`                                     | [x]         |
| Order book for a security and board                               | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/orderbook`                | [x]         |
| Trades for a security and board                                   | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/trades`                   | [x]         |
| Candles for a security and board                                  | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/candles`                  | [x]         |
| Stock market indexes                                              | `/iss/statistics/engines/stock/markets/index/analytics`                                                | [x]         |
| Index constituents                                                | `/iss/statistics/engines/stock/markets/index/analytics/[indexid]`                                      | [x]         |
| All MOEX securities                                               | `/iss/securities`                                                                                      | [x]         |
| Security details by `secid`                                       | `/iss/securities/[security]`                                                                           | [x]         |
| Securities at market level (without `boards/[board]`)             | `/iss/engines/[engine]/markets/[market]/securities`                                                    | [x]         |
| Trades and order books at market level (without `boards/[board]`) | `/iss/engines/[engine]/markets/[market]/trades`, `/orderbook`                                          | [x]         |
| Candle borders for a security                                     | `/iss/engines/[engine]/markets/[market]/securities/[security]/candleborders`                           | [x]         |
| Available history dates for a security                            | `/iss/history/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/dates`            | [x]         |
| Historical data                                                   | `/iss/history/...`                                                                                     | [x]         |
| Turnovers and security statistics                                 | `/iss/turnovers`, `/iss/engines/[engine]/turnovers`, `/iss/engines/[engine]/markets/[market]/secstats` | [x]         |
| ReferenceData 2.0                                                 | `/iss/referencedata/...`                                                                               | [ ]         |
| News and events                                                   | `/iss/sitenews`, `/iss/events`                                                                         | [x]         |

## Feature flags

The default features are `blocking` and `rustls-tls`.

- `async` — asynchronous client.
- `blocking` — blocking client.
- `history` — `/history/...` endpoints.
- `news` — `/sitenews` and `/events` endpoints.
- `rustls-tls` / `native-tls` — the TLS backend used by `reqwest`.

Async-only configuration:

```toml
moex-client = { version = "...", default-features = false, features = ["async", "rustls-tls"] }
```

For a blocking-only configuration, replace `async` with `blocking`.

## Quick start

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

The asynchronous client has the same API with async request methods:

```rust
use moex_client::r#async::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().user_agent_from_crate().build()?;
    let engines = client.engines().await?;
    println!("engines: {}", engines.len());
    Ok(())
}
```

To run the included index export example:

```bash
cargo run --example actual_indexes_dump
```

## Proxy

Configure a proxy through the client builder:

```rust
use moex_client::blocking::Client;

let proxy = reqwest::Proxy::all("http://127.0.0.1:3128")?;
let _client = Client::builder().proxy(proxy).build()?;
```

Use `Client::builder().no_proxy()` to ignore environment and system proxy settings. The asynchronous client exposes the same builder methods.

## Retries and rate limiting

`with_retry` retries transport operations according to a caller-provided policy:

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

A rate limit can be attached to a blocking client and applied to every request:

```rust
use std::time::Duration;

use moex_client::RateLimit;
use moex_client::blocking::Client;

let _client = Client::builder()
    .rate_limit(RateLimit::every(Duration::from_millis(250)))
    .build()?;
```

For asynchronous code, use `with_retry_async` and `with_rate_limit_async`. They accept the runtime's sleep function, such as `tokio::time::sleep`. A rate-limited async client likewise requires `.rate_limit_sleep(tokio::time::sleep)`.

## Fluent selectors and prelude

The `Client` API maps directly to endpoints; extension traits such as `IndexesExt`, `IndexAnalyticsExt`, and `SecurityBoardsExt` provide domain-level collection operations. Import them together with:

```rust
use moex_client::prelude::*;
```

```rust
use std::num::NonZeroU32;

use moex_client::blocking::Client;
use moex_client::models::{IndexId, PageRequest, SecId};
use moex_client::prelude::*;

fn demo(client: &Client, indexid: &IndexId, secid: &SecId) -> Result<(), moex_client::MoexError> {
    let _indexes = client.indexes()?.into_actual_by_till();
    let _components = client
        .index(indexid.clone())?
        .analytics(PageRequest::all(NonZeroU32::new(5000).expect("non-zero")))?
        .into_actual_by_session()
        .into_sorted_by_weight_desc();
    let _board = client
        .security(secid.clone())?
        .boards()?
        .into_stock_primary_or_first();
    Ok(())
}
```

## Raw ISS requests

The low-level builder covers endpoints that do not yet have strict methods:

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

When an endpoint has a typed representation, it can be used without assembling its path manually:

```rust
use moex_client::blocking::Client;
use moex_client::models::{BoardId, EngineName, MarketName};
use moex_client::{IssEndpoint, MoexError};

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

`path` accepts `engines`, `engines.json`, `/iss/engines`, and `/iss/engines.json`, normalizing them to a relative `<endpoint>.json`. Query strings must be supplied through `param`. Typed `IssRequestOptions` configure system `iss.*` parameters, while `send_response` returns the complete HTTP response and `send_table` decodes one table into a caller-defined type.

The `moex_client::decode` module also decodes payloads without a client. It includes endpoint-specific functions, `raw_table_rows_json` for an owned user-defined row type, `raw_tables_json` for extracting several tables after parsing the top level once, and `raw_table_view_json` for borrowed cell access.

## Scoped API and pagination

`PageRequest` provides consistent pagination for index analytics, securities, trades, candles, and history. Fluent scopes bind `engine`, `market`, `board`, and `security` once. Lazy paginators support both `next_page` and `all`.

For history requests, `HistoryQuery` adds server-side `from` and `till` bounds. Market-specific `DURATION` and `YIELD` values are available through `HistoryRecord::duration_days` and `HistoryRecord::yield_percent`; either method returns `None` when the market response omits the corresponding column.

```rust
use std::num::NonZeroU32;

use moex_client::blocking::Client;
use moex_client::models::{BoardId, EngineName, MarketName, PageRequest, SecId};

fn demo_scoped(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let engine = EngineName::try_from("stock")?;
    let market = MarketName::try_from("shares")?;
    let board = BoardId::try_from("TQBR")?;
    let secid = SecId::try_from("SBER")?;

    let security = client
        .engine(engine)?
        .market(market)?
        .board(board)?
        .security(secid)?;

    let _history = security.history(PageRequest::first_page())?;
    let _all_trades = security
        .trades_pages(NonZeroU32::new(1000).expect("non-zero"))
        .all()?;
    Ok(())
}
```

# moex-client

[🇺🇸 English](./README.md) · [🇷🇺 Русский](./README.ru.md)

[![CI](https://github.com/hexqnt/moex-client/actions/workflows/ci.yml/badge.svg)](https://github.com/hexqnt/moex-client/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/moex-client.svg)](https://crates.io/crates/moex-client)
[![docs.rs](https://docs.rs/moex-client/badge.svg)](https://docs.rs/moex-client)

Неофициальная строго типизированная библиотека на Rust для работы с ISS API Московской биржи.

## Принципы API

- Публичный API работает со строгими доменными типами (`SecId`, `BoardId`, `EngineName`, `MarketName`, `IndexId`).
- Данные ISS валидируются на границе и преобразуются в модели через `TryFrom` и `try_new`.
- Extension traits обеспечивают fluent-обработку коллекций (`moex_client::prelude::*`).
- Эндпоинты без строгого API доступны через raw-конструктор запросов с типизированными опциями `iss.*`.

## Покрытие ISS

Справочник эндпоинтов: <https://iss.moex.com/iss/reference/>

| Данные                                                  | Эндпоинт ISS                                                                                           | Реализовано |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ----------- |
| Список торговых систем                                  | `/iss/engines`                                                                                         | [x]         |
| Рынки торговой системы                                  | `/iss/engines/[engine]/markets`                                                                        | [x]         |
| Режимы торгов                                           | `/iss/engines/[engine]/markets/[market]/boards`                                                        | [x]         |
| Инструменты по режиму торгов                            | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities`                                     | [x]         |
| Стакан по инструменту и режиму                          | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/orderbook`                | [x]         |
| Сделки по инструменту и режиму                          | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/trades`                   | [x]         |
| Свечи по инструменту и режиму                           | `/iss/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/candles`                  | [x]         |
| Индексы фондового рынка                                 | `/iss/statistics/engines/stock/markets/index/analytics`                                                | [x]         |
| Состав индекса                                          | `/iss/statistics/engines/stock/markets/index/analytics/[indexid]`                                      | [x]         |
| Все бумаги MOEX                                         | `/iss/securities`                                                                                      | [x]         |
| Карточка бумаги по `secid`                              | `/iss/securities/[security]`                                                                           | [x]         |
| Инструменты на уровне рынка (без `boards/[board]`)      | `/iss/engines/[engine]/markets/[market]/securities`                                                    | [x]         |
| Сделки и стаканы на уровне рынка (без `boards/[board]`) | `/iss/engines/[engine]/markets/[market]/trades`, `/orderbook`                                          | [x]         |
| Границы свечей по инструменту                           | `/iss/engines/[engine]/markets/[market]/securities/[security]/candleborders`                           | [x]         |
| Доступные даты истории по инструменту                   | `/iss/history/engines/[engine]/markets/[market]/boards/[board]/securities/[security]/dates`            | [x]         |
| Исторические данные                                     | `/iss/history/...`                                                                                     | [x]         |
| Обороты и статистика по бумагам                         | `/iss/turnovers`, `/iss/engines/[engine]/turnovers`, `/iss/engines/[engine]/markets/[market]/secstats` | [x]         |
| ReferenceData 2.0                                       | `/iss/referencedata/...`                                                                               | [ ]         |
| Новости и события                                       | `/iss/sitenews`, `/iss/events`                                                                         | [x]         |

## Feature flags

По умолчанию включены `blocking` и `rustls-tls`.

- `async` — асинхронный клиент.
- `blocking` — блокирующий клиент.
- `history` — эндпоинты `/history/...`.
- `news` — эндпоинты `/sitenews` и `/events`.
- `rustls-tls` / `native-tls` — TLS-бэкенд для `reqwest`.

Конфигурация только с асинхронным API:

```toml
moex-client = { version = "...", default-features = false, features = ["async", "rustls-tls"] }
```

Для конфигурации только с блокирующим API замените `async` на `blocking`.

## Быстрый старт

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

Асинхронный клиент имеет такой же API с асинхронными методами запросов:

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

Запуск примера выгрузки индексов из репозитория:

```bash
cargo run --example actual_indexes_dump
```

## Proxy

Proxy настраивается через builder клиента:

```rust
use moex_client::blocking::Client;

let proxy = reqwest::Proxy::all("http://127.0.0.1:3128")?;
let _client = Client::builder().proxy(proxy).build()?;
```

`Client::builder().no_proxy()` отключает proxy из окружения и системных настроек. Асинхронный клиент предоставляет те же методы builder-а.

## Повторные попытки и ограничение частоты запросов

`with_retry` повторяет транспортные операции согласно переданной политике:

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

Ограничение частоты можно привязать к блокирующему клиенту и применять ко всем запросам:

```rust
use std::time::Duration;

use moex_client::RateLimit;
use moex_client::blocking::Client;

let _client = Client::builder()
    .rate_limit(RateLimit::every(Duration::from_millis(250)))
    .build()?;
```

В асинхронном коде используйте `with_retry_async` и `with_rate_limit_async`. Они принимают функцию ожидания из runtime, например `tokio::time::sleep`. Асинхронному клиенту с ограничением частоты также требуется `.rate_limit_sleep(tokio::time::sleep)`.

## Fluent-селекторы и prelude

API `Client` напрямую соответствует эндпоинтам, а extension traits `IndexesExt`, `IndexAnalyticsExt` и `SecurityBoardsExt` предоставляют операции над доменными коллекциями. Для их общего импорта:

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

## Raw-запросы ISS

Низкоуровневый builder покрывает эндпоинты, для которых ещё нет строгих методов:

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

Если эндпоинт имеет типизированное представление, его можно использовать без ручной сборки пути:

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

`path` принимает `engines`, `engines.json`, `/iss/engines` и `/iss/engines.json`, нормализуя их в относительный `<endpoint>.json`. Query-строку нужно передавать через `param`. Типизированные `IssRequestOptions` настраивают системные параметры `iss.*`, `send_response` возвращает полный HTTP-ответ, а `send_table` декодирует одну таблицу в пользовательский тип.

Модуль `moex_client::decode` также декодирует payload без клиента. Он содержит специализированные для эндпоинтов функции, `raw_table_rows_json` для пользовательского типа строки с владением данными, `raw_tables_json` для извлечения нескольких таблиц после однократного разбора верхнего уровня и `raw_table_view_json` для заимствованного доступа к ячейкам.

## Scoped API и пагинация

`PageRequest` унифицирует пагинацию для аналитики индексов, инструментов, сделок, свечей и истории. Fluent-scopes однократно связывают `engine`, `market`, `board` и `security`. Ленивые пагинаторы поддерживают `next_page` и `all`.

Для запросов истории `HistoryQuery` добавляет серверные границы `from` и `till`. Специфичные для рынка значения `DURATION` и `YIELD` доступны через `HistoryRecord::duration_days` и `HistoryRecord::yield_percent`; каждый метод возвращает `None`, если в ответе рынка нет соответствующей колонки.

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

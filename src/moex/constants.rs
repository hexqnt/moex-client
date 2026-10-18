//! Централизованные константы wire-слоя ISS и шаблоны endpoint-ов.

#[cfg(any(feature = "async", feature = "blocking"))]
mod query;

use crate::models::{BoardId, EngineName, IndexId, MarketName, SecId};

#[cfg(any(feature = "async", feature = "blocking"))]
pub(super) use query::*;

// Базовый URL и endpoint-шаблоны ISS.
#[cfg(any(feature = "async", feature = "blocking"))]
pub(super) const BASE_URL: &str = "https://iss.moex.com/iss/";
pub(super) const INDEXES_ENDPOINT: &str = "statistics/engines/stock/markets/index/analytics.json";
pub(super) const INDEX_ANALYTICS_ENDPOINT_TEMPLATE: &str =
    "statistics/engines/stock/markets/index/analytics/{index}.json";
pub(super) const TURNOVERS_ENDPOINT: &str = "turnovers.json";
#[cfg(feature = "news")]
pub(super) const SITENEWS_ENDPOINT: &str = "sitenews.json";
#[cfg(feature = "news")]
pub(super) const EVENTS_ENDPOINT: &str = "events.json";
#[cfg(feature = "history")]
pub(super) const HISTORY_DATES_ENDPOINT_TEMPLATE: &str =
    "history/engines/{engine}/markets/{market}/boards/{board}/securities/{security}/dates.json";
#[cfg(feature = "history")]
pub(super) const HISTORY_ENDPOINT_TEMPLATE: &str =
    "history/engines/{engine}/markets/{market}/boards/{board}/securities/{security}.json";
pub(super) const SECSTATS_ENDPOINT_TEMPLATE: &str =
    "engines/{engine}/markets/{market}/secstats.json";
pub(super) const GLOBAL_SECURITIES_ENDPOINT: &str = "securities.json";
pub(super) const SECURITY_BOARDS_ENDPOINT_TEMPLATE: &str = "securities/{security}.json";
pub(super) const ENGINES_ENDPOINT: &str = "engines.json";
pub(super) const MARKETS_ENDPOINT_TEMPLATE: &str = "engines/{engine}/markets.json";
pub(super) const BOARDS_ENDPOINT_TEMPLATE: &str = "engines/{engine}/markets/{market}/boards.json";
pub(super) const SECURITIES_ENDPOINT_TEMPLATE: &str =
    "engines/{engine}/markets/{market}/boards/{board}/securities.json";
pub(super) const ORDERBOOK_ENDPOINT_TEMPLATE: &str =
    "engines/{engine}/markets/{market}/boards/{board}/securities/{security}/orderbook.json";
pub(super) const CANDLEBORDERS_ENDPOINT_TEMPLATE: &str =
    "engines/{engine}/markets/{market}/securities/{security}/candleborders.json";
pub(super) const CANDLES_ENDPOINT_TEMPLATE: &str =
    "engines/{engine}/markets/{market}/boards/{board}/securities/{security}/candles.json";
pub(super) const TRADES_ENDPOINT_TEMPLATE: &str =
    "engines/{engine}/markets/{market}/boards/{board}/securities/{security}/trades.json";

/// Построить endpoint для `engines/{engine}/markets.json`.
pub(super) fn markets_endpoint(engine: &EngineName) -> String {
    format!("engines/{}/markets.json", engine.as_str())
}

/// Построить endpoint для `index_analytics` выбранного индекса.
pub(super) fn index_analytics_endpoint(indexid: &IndexId) -> String {
    format!(
        "statistics/engines/stock/markets/index/analytics/{}.json",
        indexid.as_str()
    )
}

pub(super) fn engine_turnovers_endpoint(engine: &EngineName) -> String {
    format!("engines/{}/turnovers.json", engine.as_str())
}

/// Построить endpoint ресурса `securities/{security}.json`.
pub(super) fn security_endpoint(security: &SecId) -> String {
    format!("securities/{}.json", security.as_str())
}

#[cfg(feature = "history")]
pub(super) fn history_dates_endpoint(
    engine: &EngineName,
    market: &MarketName,
    board: &BoardId,
    security: &SecId,
) -> String {
    format!(
        "history/engines/{}/markets/{}/boards/{}/securities/{}/dates.json",
        engine.as_str(),
        market.as_str(),
        board.as_str(),
        security.as_str()
    )
}

#[cfg(feature = "history")]
pub(super) fn history_endpoint(
    engine: &EngineName,
    market: &MarketName,
    board: &BoardId,
    security: &SecId,
) -> String {
    format!(
        "history/engines/{}/markets/{}/boards/{}/securities/{}.json",
        engine.as_str(),
        market.as_str(),
        board.as_str(),
        security.as_str()
    )
}

pub(super) fn secstats_endpoint(engine: &EngineName, market: &MarketName) -> String {
    format!(
        "engines/{}/markets/{}/secstats.json",
        engine.as_str(),
        market.as_str()
    )
}

pub(super) fn boards_endpoint(engine: &EngineName, market: &MarketName) -> String {
    format!(
        "engines/{}/markets/{}/boards.json",
        engine.as_str(),
        market.as_str()
    )
}

pub(super) fn market_securities_endpoint(engine: &EngineName, market: &MarketName) -> String {
    format!(
        "engines/{}/markets/{}/securities.json",
        engine.as_str(),
        market.as_str()
    )
}

pub(super) fn market_security_endpoint(
    engine: &EngineName,
    market: &MarketName,
    security: &SecId,
) -> String {
    format!(
        "engines/{}/markets/{}/securities/{}.json",
        engine.as_str(),
        market.as_str(),
        security.as_str()
    )
}

pub(super) fn market_orderbook_endpoint(engine: &EngineName, market: &MarketName) -> String {
    format!(
        "engines/{}/markets/{}/orderbook.json",
        engine.as_str(),
        market.as_str()
    )
}

pub(super) fn market_trades_endpoint(engine: &EngineName, market: &MarketName) -> String {
    format!(
        "engines/{}/markets/{}/trades.json",
        engine.as_str(),
        market.as_str()
    )
}

pub(super) fn candleborders_endpoint(
    engine: &EngineName,
    market: &MarketName,
    security: &SecId,
) -> String {
    format!(
        "engines/{}/markets/{}/securities/{}/candleborders.json",
        engine.as_str(),
        market.as_str(),
        security.as_str()
    )
}

pub(super) fn securities_endpoint(
    engine: &EngineName,
    market: &MarketName,
    board: &BoardId,
) -> String {
    format!(
        "engines/{}/markets/{}/boards/{}/securities.json",
        engine.as_str(),
        market.as_str(),
        board.as_str()
    )
}

pub(super) fn orderbook_endpoint(
    engine: &EngineName,
    market: &MarketName,
    board: &BoardId,
    security: &SecId,
) -> String {
    format!(
        "engines/{}/markets/{}/boards/{}/securities/{}/orderbook.json",
        engine.as_str(),
        market.as_str(),
        board.as_str(),
        security.as_str()
    )
}

pub(super) fn candles_endpoint(
    engine: &EngineName,
    market: &MarketName,
    board: &BoardId,
    security: &SecId,
) -> String {
    format!(
        "engines/{}/markets/{}/boards/{}/securities/{}/candles.json",
        engine.as_str(),
        market.as_str(),
        board.as_str(),
        security.as_str()
    )
}

pub(super) fn trades_endpoint(
    engine: &EngineName,
    market: &MarketName,
    board: &BoardId,
    security: &SecId,
) -> String {
    format!(
        "engines/{}/markets/{}/boards/{}/securities/{}/trades.json",
        engine.as_str(),
        market.as_str(),
        board.as_str(),
        security.as_str()
    )
}

#[cfg(any(feature = "async", feature = "blocking"))]
pub(super) fn metadata_value(metadata: bool) -> &'static str {
    if metadata { "on" } else { "off" }
}

//! Централизованные константы wire-слоя ISS и шаблоны endpoint-ов.

#[cfg(any(feature = "async", feature = "blocking"))]
mod query;

use crate::models::{BoardId, EngineName, IndexId, MarketName, SecId};

#[cfg(any(feature = "async", feature = "blocking"))]
pub(super) use query::*;

#[cfg(any(feature = "async", feature = "blocking"))]
pub(super) const BASE_URL: &str = "https://iss.moex.com/iss/";
pub(super) const INDEXES_ENDPOINT: &str = "statistics/engines/stock/markets/index/analytics.json";
pub(super) const TURNOVERS_ENDPOINT: &str = "turnovers.json";
pub(super) const GLOBAL_SECURITIES_ENDPOINT: &str = "securities.json";
pub(super) const ENGINES_ENDPOINT: &str = "engines.json";
#[cfg(feature = "news")]
pub(super) const SITENEWS_ENDPOINT: &str = "sitenews.json";
#[cfg(feature = "news")]
pub(super) const EVENTS_ENDPOINT: &str = "events.json";

// Один литерал задаёт путь запроса и шаблон для ошибок автономного декодера.
macro_rules! endpoints {
    ($($(#[$attr:meta])* $name:ident($($arg:ident: $ty:ty),*) $(=> $template:ident)? = $path:literal;)*) => {
        $(
            endpoints!(@template [$(#[$attr])*] [$($template)?] $path);

            $(#[$attr])*
            pub(super) fn $name($($arg: &$ty),*) -> String {
                format!($path, $($arg = $arg.as_str()),*)
            }
        )*
    };
    (@template [$($attr:tt)*] [$template:ident] $path:literal) => {
        $($attr)*
        pub(super) const $template: &str = $path;
    };
    (@template [$($attr:tt)*] [] $path:literal) => {};
}

endpoints! {
    markets_endpoint(engine: EngineName) => MARKETS_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets.json";
    index_analytics_endpoint(index: IndexId) => INDEX_ANALYTICS_ENDPOINT_TEMPLATE =
        "statistics/engines/stock/markets/index/analytics/{index}.json";
    engine_turnovers_endpoint(engine: EngineName) =
        "engines/{engine}/turnovers.json";
    security_endpoint(security: SecId) => SECURITY_BOARDS_ENDPOINT_TEMPLATE =
        "securities/{security}.json";
    #[cfg(feature = "history")]
    history_dates_endpoint(engine: EngineName, market: MarketName, board: BoardId, security: SecId) => HISTORY_DATES_ENDPOINT_TEMPLATE =
        "history/engines/{engine}/markets/{market}/boards/{board}/securities/{security}/dates.json";
    #[cfg(feature = "history")]
    history_endpoint(engine: EngineName, market: MarketName, board: BoardId, security: SecId) => HISTORY_ENDPOINT_TEMPLATE =
        "history/engines/{engine}/markets/{market}/boards/{board}/securities/{security}.json";
    secstats_endpoint(engine: EngineName, market: MarketName) => SECSTATS_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets/{market}/secstats.json";
    boards_endpoint(engine: EngineName, market: MarketName) => BOARDS_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets/{market}/boards.json";
    market_securities_endpoint(engine: EngineName, market: MarketName) =
        "engines/{engine}/markets/{market}/securities.json";
    market_security_endpoint(engine: EngineName, market: MarketName, security: SecId) =
        "engines/{engine}/markets/{market}/securities/{security}.json";
    market_orderbook_endpoint(engine: EngineName, market: MarketName) =
        "engines/{engine}/markets/{market}/orderbook.json";
    market_trades_endpoint(engine: EngineName, market: MarketName) =
        "engines/{engine}/markets/{market}/trades.json";
    candleborders_endpoint(engine: EngineName, market: MarketName, security: SecId) => CANDLEBORDERS_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets/{market}/securities/{security}/candleborders.json";
    securities_endpoint(engine: EngineName, market: MarketName, board: BoardId) => SECURITIES_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets/{market}/boards/{board}/securities.json";
    orderbook_endpoint(engine: EngineName, market: MarketName, board: BoardId, security: SecId) => ORDERBOOK_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets/{market}/boards/{board}/securities/{security}/orderbook.json";
    candles_endpoint(engine: EngineName, market: MarketName, board: BoardId, security: SecId) => CANDLES_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets/{market}/boards/{board}/securities/{security}/candles.json";
    trades_endpoint(engine: EngineName, market: MarketName, board: BoardId, security: SecId) => TRADES_ENDPOINT_TEMPLATE =
        "engines/{engine}/markets/{market}/boards/{board}/securities/{security}/trades.json";
}

#[cfg(any(feature = "async", feature = "blocking"))]
pub(super) fn metadata_value(metadata: bool) -> &'static str {
    super::request::IssToggle::from(metadata).as_query_value()
}

//! Централизованные константы wire-слоя ISS и шаблоны endpoint-ов.

use crate::models::{BoardId, EngineName, IndexId, MarketName, SecId};

// Базовый URL и endpoint-шаблоны ISS.
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

// Системные query-параметры ISS.
pub(super) const ISS_META_PARAM: &str = "iss.meta";
pub(super) const ISS_DATA_PARAM: &str = "iss.data";
pub(super) const ISS_JSON_PARAM: &str = "iss.json";
pub(super) const ISS_VERSION_PARAM: &str = "iss.version";
pub(super) const ISS_ONLY_PARAM: &str = "iss.only";

// Параметры выбора колонок и фиксированные наборы колонок.
pub(super) const INDICES_COLUMNS_PARAM: &str = "indices.columns";
pub(super) const INDICES_COLUMNS: &str = "indexid,shortname,from,till";
pub(super) const ANALYTICS_COLUMNS_PARAM: &str = "analytics.columns";
pub(super) const ANALYTICS_COLUMNS: &str =
    "indexid,tradedate,ticker,shortnames,secids,weight,tradingsession,trade_session_date";
pub(super) const ENGINES_COLUMNS_PARAM: &str = "engines.columns";
pub(super) const ENGINES_COLUMNS: &str = "id,name,title";
pub(super) const MARKETS_COLUMNS_PARAM: &str = "markets.columns";
pub(super) const MARKETS_COLUMNS: &str = "id,NAME,title";
pub(super) const BOARDS_COLUMNS_PARAM: &str = "boards.columns";
pub(super) const BOARDS_COLUMNS: &str = "id,board_group_id,boardid,title,is_traded";
pub(super) const SECURITY_BOARDS_COLUMNS: &str = "engine,market,boardid,is_primary";
pub(super) const SECURITIES_COLUMNS_PARAM: &str = "securities.columns";
pub(super) const SECURITIES_COLUMNS: &str = "SECID,SHORTNAME,SECNAME,STATUS";
pub(super) const SECURITIES_SNAPSHOT_COLUMNS: &str = "SECID,LOTSIZE";
pub(super) const MARKETDATA_COLUMNS_PARAM: &str = "marketdata.columns";
pub(super) const MARKETDATA_LAST_COLUMNS: &str = "SECID,LAST";
pub(super) const ORDERBOOK_COLUMNS_PARAM: &str = "orderbook.columns";
pub(super) const ORDERBOOK_COLUMNS: &str = "BUYSELL,PRICE,QUANTITY";
pub(super) const CANDLES_COLUMNS_PARAM: &str = "candles.columns";
pub(super) const CANDLES_COLUMNS: &str = "begin,end,open,close,high,low,value,volume";
pub(super) const TRADES_COLUMNS_PARAM: &str = "trades.columns";
pub(super) const TRADES_COLUMNS: &str = "TRADENO,TRADETIME,PRICE,QUANTITY,VALUE";
pub(super) const TURNOVERS_COLUMNS_PARAM: &str = "turnovers.columns";
pub(super) const TURNOVERS_COLUMNS: &str =
    "NAME,ID,VALTODAY,VALTODAY_USD,NUMTRADES,UPDATETIME,TITLE";
#[cfg(feature = "news")]
pub(super) const SITENEWS_COLUMNS_PARAM: &str = "sitenews.columns";
#[cfg(feature = "news")]
pub(super) const SITENEWS_COLUMNS: &str = "id,tag,title,published_at,modified_at";
#[cfg(feature = "news")]
pub(super) const EVENTS_COLUMNS_PARAM: &str = "events.columns";
#[cfg(feature = "news")]
pub(super) const EVENTS_COLUMNS: &str = "id,tag,title,from,modified_at";
pub(super) const SECSTATS_COLUMNS_PARAM: &str = "secstats.columns";
pub(super) const SECSTATS_COLUMNS: &str = "SECID,BOARDID,VOLTODAY,VALTODAY,HIGHBID,LOWOFFER,LASTOFFER,LASTBID,OPEN,LOW,HIGH,LAST,NUMTRADES,WAPRICE";
#[cfg(feature = "history")]
pub(super) const HISTORY_COLUMNS_PARAM: &str = "history.columns";
#[cfg(feature = "history")]
pub(super) const HISTORY_COLUMNS: &str =
    "BOARDID,TRADEDATE,SECID,NUMTRADES,VALUE,OPEN,LOW,HIGH,CLOSE,VOLUME";

// Общие query-параметры пагинации/фильтрации.
pub(super) const FROM_PARAM: &str = "from";
pub(super) const TILL_PARAM: &str = "till";
pub(super) const INTERVAL_PARAM: &str = "interval";
pub(super) const START_PARAM: &str = "start";
pub(super) const LIMIT_PARAM: &str = "limit";
pub(super) const NON_JSON_BODY_PREFIX_CHARS: usize = 180;

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

pub(super) fn security_boards_endpoint(security: &SecId) -> String {
    security_endpoint(security)
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

pub(super) fn metadata_value(metadata: bool) -> &'static str {
    if metadata { "on" } else { "off" }
}

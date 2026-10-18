//! Параметры транспортных запросов и фиксированные наборы колонок ISS.

// Системные query-параметры ISS.
pub(in crate::moex) const ISS_META_PARAM: &str = "iss.meta";
pub(in crate::moex) const ISS_DATA_PARAM: &str = "iss.data";
pub(in crate::moex) const ISS_JSON_PARAM: &str = "iss.json";
pub(in crate::moex) const ISS_VERSION_PARAM: &str = "iss.version";
pub(in crate::moex) const ISS_ONLY_PARAM: &str = "iss.only";

// Параметры выбора колонок и фиксированные наборы колонок.
pub(in crate::moex) const INDICES_COLUMNS_PARAM: &str = "indices.columns";
pub(in crate::moex) const INDICES_COLUMNS: &str = "indexid,shortname,from,till";
pub(in crate::moex) const ANALYTICS_COLUMNS_PARAM: &str = "analytics.columns";
pub(in crate::moex) const ANALYTICS_COLUMNS: &str =
    "indexid,tradedate,ticker,shortnames,secids,weight,tradingsession,trade_session_date";
pub(in crate::moex) const ENGINES_COLUMNS_PARAM: &str = "engines.columns";
pub(in crate::moex) const ENGINES_COLUMNS: &str = "id,name,title";
pub(in crate::moex) const MARKETS_COLUMNS_PARAM: &str = "markets.columns";
pub(in crate::moex) const MARKETS_COLUMNS: &str = "id,NAME,title";
pub(in crate::moex) const BOARDS_COLUMNS_PARAM: &str = "boards.columns";
pub(in crate::moex) const BOARDS_COLUMNS: &str = "id,board_group_id,boardid,title,is_traded";
pub(in crate::moex) const SECURITY_BOARDS_COLUMNS: &str = "engine,market,boardid,is_primary";
pub(in crate::moex) const SECURITIES_COLUMNS_PARAM: &str = "securities.columns";
pub(in crate::moex) const SECURITIES_COLUMNS: &str = "SECID,SHORTNAME,SECNAME,STATUS";
pub(in crate::moex) const SECURITIES_SNAPSHOT_COLUMNS: &str = "SECID,LOTSIZE";
pub(in crate::moex) const MARKETDATA_COLUMNS_PARAM: &str = "marketdata.columns";
pub(in crate::moex) const MARKETDATA_LAST_COLUMNS: &str = "SECID,LAST";
pub(in crate::moex) const ORDERBOOK_COLUMNS_PARAM: &str = "orderbook.columns";
pub(in crate::moex) const ORDERBOOK_COLUMNS: &str = "BUYSELL,PRICE,QUANTITY";
pub(in crate::moex) const CANDLES_COLUMNS_PARAM: &str = "candles.columns";
pub(in crate::moex) const CANDLES_COLUMNS: &str = "begin,end,open,close,high,low,value,volume";
pub(in crate::moex) const TRADES_COLUMNS_PARAM: &str = "trades.columns";
pub(in crate::moex) const TRADES_COLUMNS: &str = "TRADENO,TRADETIME,PRICE,QUANTITY,VALUE";
pub(in crate::moex) const TURNOVERS_COLUMNS_PARAM: &str = "turnovers.columns";
pub(in crate::moex) const TURNOVERS_COLUMNS: &str =
    "NAME,ID,VALTODAY,VALTODAY_USD,NUMTRADES,UPDATETIME,TITLE";
#[cfg(feature = "news")]
pub(in crate::moex) const SITENEWS_COLUMNS_PARAM: &str = "sitenews.columns";
#[cfg(feature = "news")]
pub(in crate::moex) const SITENEWS_COLUMNS: &str = "id,tag,title,published_at,modified_at";
#[cfg(feature = "news")]
pub(in crate::moex) const EVENTS_COLUMNS_PARAM: &str = "events.columns";
#[cfg(feature = "news")]
pub(in crate::moex) const EVENTS_COLUMNS: &str = "id,tag,title,from,modified_at";
pub(in crate::moex) const SECSTATS_COLUMNS_PARAM: &str = "secstats.columns";
pub(in crate::moex) const SECSTATS_COLUMNS: &str = "SECID,BOARDID,VOLTODAY,VALTODAY,HIGHBID,LOWOFFER,LASTOFFER,LASTBID,OPEN,LOW,HIGH,LAST,NUMTRADES,WAPRICE";
#[cfg(feature = "history")]
pub(in crate::moex) const HISTORY_COLUMNS_PARAM: &str = "history.columns";
#[cfg(feature = "history")]
pub(in crate::moex) const HISTORY_COLUMNS: &str =
    "BOARDID,TRADEDATE,SECID,NUMTRADES,VALUE,OPEN,LOW,HIGH,CLOSE,VOLUME,DURATION,YIELD";

// Общие query-параметры пагинации/фильтрации.
pub(in crate::moex) const FROM_PARAM: &str = "from";
pub(in crate::moex) const TILL_PARAM: &str = "till";
pub(in crate::moex) const INTERVAL_PARAM: &str = "interval";
pub(in crate::moex) const START_PARAM: &str = "start";
pub(in crate::moex) const LIMIT_PARAM: &str = "limit";
pub(in crate::moex) const NON_JSON_BODY_PREFIX_CHARS: usize = 180;

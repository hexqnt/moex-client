//! Параметры транспортных запросов и фиксированные наборы колонок ISS.

// Системные query-параметры ISS.
pub(in crate::moex) const ISS_META_PARAM: &str = "iss.meta";
pub(in crate::moex) const ISS_DATA_PARAM: &str = "iss.data";
pub(in crate::moex) const ISS_JSON_PARAM: &str = "iss.json";
pub(in crate::moex) const ISS_VERSION_PARAM: &str = "iss.version";
pub(in crate::moex) const ISS_ONLY_PARAM: &str = "iss.only";

// Описание выборки хранит имя таблицы вместе с её колонками, сохраняя порядок query-пар.
macro_rules! table_selections {
    ($($(#[$attr:meta])* $name:ident $(($only:literal))? { $($table:ident: $columns:literal),* $(,)? };)*) => {
        #[derive(Clone, Copy)]
        pub(in crate::moex) enum TableSelection {
            $($(#[$attr])* $name,)*
        }

        impl TableSelection {
            pub(in crate::moex) fn parameters(self) -> &'static [(&'static str, &'static str)] {
                match self {
                    $($(#[$attr])* Self::$name => &[
                        $((ISS_ONLY_PARAM, $only),)?
                        $((concat!(stringify!($table), ".columns"), $columns),)*
                    ],)*
                }
            }
        }
    };
}

table_selections! {
    Indices("indices") { indices: "indexid,shortname,from,till" };
    Analytics("analytics") { analytics: "indexid,tradedate,ticker,shortnames,secids,weight,tradingsession,trade_session_date" };
    Turnovers("turnovers") { turnovers: "NAME,ID,VALTODAY,VALTODAY_USD,NUMTRADES,UPDATETIME,TITLE" };
    Engines("engines") { engines: "id,name,title" };
    Markets("markets") { markets: "id,NAME,title" };
    Boards("boards") { boards: "id,board_group_id,boardid,title,is_traded" };
    SecurityBoards("boards") { boards: "engine,market,boardid,is_primary" };
    Securities("securities") { securities: "SECID,SHORTNAME,SECNAME,STATUS" };
    Snapshots("securities,marketdata") {
        securities: "SECID,LOTSIZE",
        marketdata: "SECID,LAST",
    };
    Orderbook("orderbook") { orderbook: "BUYSELL,PRICE,QUANTITY" };
    Candles("candles") { candles: "begin,end,open,close,high,low,value,volume" };
    Trades("trades") { trades: "TRADENO,TRADETIME,PRICE,QUANTITY,VALUE" };
    SecStats("secstats") { secstats: "SECID,BOARDID,VOLTODAY,VALTODAY,HIGHBID,LOWOFFER,LASTOFFER,LASTBID,OPEN,LOW,HIGH,LAST,NUMTRADES,WAPRICE" };
    MetadataOnly {};
    #[cfg(feature = "news")]
    SiteNews("sitenews") { sitenews: "id,tag,title,published_at,modified_at" };
    #[cfg(feature = "news")]
    Events("events") { events: "id,tag,title,from,modified_at" };
    #[cfg(feature = "history")]
    HistoryDates("dates") {};
    #[cfg(feature = "history")]
    History("history") { history: "BOARDID,TRADEDATE,SECID,NUMTRADES,VALUE,OPEN,LOW,HIGH,CLOSE,VOLUME,DURATION,YIELD" };
}

// Общие query-параметры пагинации/фильтрации.
pub(in crate::moex) const FROM_PARAM: &str = "from";
pub(in crate::moex) const TILL_PARAM: &str = "till";
pub(in crate::moex) const INTERVAL_PARAM: &str = "interval";
pub(in crate::moex) const START_PARAM: &str = "start";
pub(in crate::moex) const LIMIT_PARAM: &str = "limit";
pub(in crate::moex) const NON_JSON_BODY_PREFIX_CHARS: usize = 180;

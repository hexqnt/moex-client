use super::*;

macro_rules! table_method {
    ($mode:ident, [$($asyncness:tt)?], $(#[$doc:meta])* $name:ident($($arg:ident: $ty:ty),*) -> $result:ty, $endpoint:expr, $table:ident, $decode:expr) => {
        $(#[$doc])*
        pub $($asyncness)? fn $name(&self, $($arg: &$ty),*) -> Result<$result, MoexError> {
            let endpoint = $endpoint;
            let endpoint: &str = endpoint.as_ref();
            let url = table_url(&self.base_url, self.metadata, endpoint, TableSelection::$table)?;
            let payload = await_result!($mode, self.fetch_payload(endpoint, url))?;
            ($decode)(&payload, endpoint)
        }
    };
}

macro_rules! table_endpoint {
    ($(#[$attr:meta])* $name:ident($($arg:ident: $ty:ty),*) -> $result:ty, $endpoint:expr, $table:ident, $decode:expr) => {
        #[cfg(feature = "blocking")]
        impl BlockingMoexClient {
            table_method!(blocking, [], $(#[$attr])* $name($($arg: $ty),*) -> $result, $endpoint, $table, $decode);
        }
        #[cfg(feature = "async")]
        impl AsyncMoexClient {
            table_method!(asynchronous, [async], $(#[$attr])* $name($($arg: $ty),*) -> $result, $endpoint, $table, $decode);
        }
    };
}

table_endpoint!(
    /// Получить список индексов из таблицы `indices`.
    indexes() -> Vec<Index>, INDEXES_ENDPOINT, Indices,
    |payload, _endpoint| decode_indexes_json_payload(payload)
);
table_endpoint!(
    /// Получить обороты ISS (`/iss/turnovers`).
    turnovers() -> Vec<Turnover>, TURNOVERS_ENDPOINT, Turnovers,
    decode_turnovers_json_with_endpoint
);
table_endpoint!(
    /// Получить обороты ISS по движку (`/iss/engines/{engine}/turnovers`).
    engine_turnovers(engine: EngineName) -> Vec<Turnover>, engine_turnovers_endpoint(engine), Turnovers,
    decode_turnovers_json_with_endpoint
);
table_endpoint!(
    /// Получить доступные торговые движки ISS (`engines`).
    engines() -> Vec<Engine>, ENGINES_ENDPOINT, Engines,
    |payload, _endpoint| decode_engines_json_payload(payload)
);
table_endpoint!(
    /// Получить рынки (`markets`) для заданного движка.
    markets(engine: EngineName) -> Vec<Market>, markets_endpoint(engine), Markets,
    decode_markets_json_with_endpoint
);
table_endpoint!(
    /// Получить режимы торгов (`boards`) для пары движок/рынок.
    boards(engine: EngineName, market: MarketName) -> Vec<Board>, boards_endpoint(engine, market), Boards,
    decode_boards_json_with_endpoint
);
table_endpoint!(
    /// Получить режимы торгов инструмента (`boards`) из endpoint `securities/{secid}`.
    security_boards(security: SecId) -> Vec<SecurityBoard>, security_endpoint(security), SecurityBoards,
    decode_security_boards_json_with_endpoint
);
table_endpoint!(
    /// Получить карточку инструмента (`securities`). Возвращает `Ok(None)`, если таблица пуста.
    security_info(security: SecId) -> Option<Security>, security_endpoint(security), Securities,
    decode_single_security
);
table_endpoint!(
    /// Получить карточку инструмента на уровне рынка. Возвращает `Ok(None)`, если таблица пуста.
    market_security_info(engine: EngineName, market: MarketName, security: SecId) -> Option<Security>, market_security_endpoint(engine, market, security), Securities,
    decode_single_security
);
table_endpoint!(
    /// Получить снимки инструментов (`LOTSIZE` и `LAST`) для режима торгов.
    board_snapshots(engine: EngineName, market: MarketName, board: BoardId) -> Vec<SecuritySnapshot>, securities_endpoint(engine, market, board), Snapshots,
    decode_board_security_snapshots_json_with_endpoint
);
table_endpoint!(
    /// Получить стакан на уровне рынка (`orderbook`) по первой странице ISS.
    market_orderbook(engine: EngineName, market: MarketName) -> Vec<OrderbookLevel>, market_orderbook_endpoint(engine, market), Orderbook,
    decode_orderbook_json_with_endpoint
);
table_endpoint!(
    /// Получить текущий стакан (`orderbook`) по инструменту.
    orderbook(engine: EngineName, market: MarketName, board: BoardId, security: SecId) -> Vec<OrderbookLevel>, orderbook_endpoint(engine, market, board, security), Orderbook,
    decode_orderbook_json_with_endpoint
);
table_endpoint!(
    /// Получить доступные границы свечей (`candleborders`) по инструменту.
    candle_borders(engine: EngineName, market: MarketName, security: SecId) -> Vec<CandleBorder>, candleborders_endpoint(engine, market, security), MetadataOnly,
    decode_candle_borders_json_with_endpoint
);
#[cfg(feature = "history")]
table_endpoint!(
    /// Получить диапазон доступных исторических дат по инструменту и board. Возвращает `Ok(None)`, если таблица пуста.
    history_dates(engine: EngineName, market: MarketName, board: BoardId, security: SecId) -> Option<HistoryDates>, history_dates_endpoint(engine, market, board, security), HistoryDates,
    decode_single_history_dates
);

fn decode_single_security(payload: &str, endpoint: &str) -> Result<Option<Security>, MoexError> {
    optional_single_security(
        endpoint,
        decode_securities_json_with_endpoint(payload, endpoint)?,
    )
}

#[cfg(feature = "history")]
fn decode_single_history_dates(
    payload: &str,
    endpoint: &str,
) -> Result<Option<HistoryDates>, MoexError> {
    optional_single_history_dates(
        endpoint,
        decode_history_dates_json_with_endpoint(payload, endpoint)?,
    )
}

macro_rules! paginated_methods {
    ($mode:ident, [$($asyncness:tt)?], $pages_type:ident, $item:ty,
     $query_method:ident, $pages_method:ident, $fetch:ident,
     args { $($arg:ident: $ty:ty),* }, query { $($query:ident: $query_ty:ty => $append_query:ident)? },
     $endpoint:expr, $table:ident, $decode:ident) => {
        /// Получить данные с единым режимом выборки страниц ISS.
        pub $($asyncness)? fn $query_method(&self, $($arg: &$ty,)* $($query: $query_ty,)? page_request: PageRequest) -> Result<Vec<$item>, MoexError> {
            let pagination = match page_request {
                PageRequest::FirstPage => Pagination::default(),
                PageRequest::Page(pagination) => pagination,
                PageRequest::All { page_limit } => return await_result!($mode, self.$pages_method($($arg,)* $($query,)? page_limit).all()),
            };
            await_result!($mode, self.$fetch($($arg,)* $($query,)? pagination))
        }

        /// Создать ленивый пагинатор страниц ISS.
        pub fn $pages_method<'a>(&'a self, $($arg: &'a $ty,)* $($query: $query_ty,)? page_limit: NonZeroU32) -> $pages_type<'a> {
            $pages_type {
                client: self,
                $($arg,)*
                $($query,)?
                pagination: PaginationTracker::new($endpoint, page_limit),
            }
        }

        pub(super) $($asyncness)? fn $fetch(&self, $($arg: &$ty,)* $($query: $query_ty,)? pagination: Pagination) -> Result<Vec<$item>, MoexError> {
            let endpoint = $endpoint;
            let endpoint: &str = endpoint.as_ref();
            let mut url = table_url(&self.base_url, self.metadata, endpoint, TableSelection::$table)?;
            $($append_query(&mut url, $query);)?
            append_pagination_to_url(&mut url, pagination);
            let payload = await_result!($mode, self.fetch_payload(endpoint, url))?;
            $decode(&payload, endpoint)
        }
    };
}

macro_rules! paginated_endpoint {
    ($blocking_pages:ident, $async_pages:ident, $item:ty,
     $query_method:ident, $pages_method:ident, $fetch:ident,
     args { $($arg:ident: $ty:ty),* }, query { $($query:ident: $query_ty:ty => $append_query:ident)? },
     $endpoint:expr, $table:ident, $decode:ident) => {
        #[cfg(feature = "blocking")]
        impl BlockingMoexClient {
            paginated_methods!(blocking, [], $blocking_pages, $item, $query_method, $pages_method, $fetch, args { $($arg: $ty),* }, query { $($query: $query_ty => $append_query)? }, $endpoint, $table, $decode);
        }
        #[cfg(feature = "async")]
        impl AsyncMoexClient {
            paginated_methods!(asynchronous, [async], $async_pages, $item, $query_method, $pages_method, $fetch, args { $($arg: $ty),* }, query { $($query: $query_ty => $append_query)? }, $endpoint, $table, $decode);
        }
    };
}

paginated_endpoint!(
    IndexAnalyticsPages,
    AsyncIndexAnalyticsPages,
    IndexAnalytics,
    index_analytics_query,
    index_analytics_pages,
    fetch_index_analytics_page,
    args { indexid: IndexId },
    query {},
    index_analytics_endpoint(indexid),
    Analytics,
    decode_index_analytics_json_with_endpoint
);
paginated_endpoint!(
    GlobalSecuritiesPages,
    AsyncGlobalSecuritiesPages,
    Security,
    global_securities_query,
    global_securities_pages,
    fetch_global_securities_page,
    args {},
    query {},
    GLOBAL_SECURITIES_ENDPOINT,
    Securities,
    decode_securities_json_with_endpoint
);
paginated_endpoint!(
    MarketSecuritiesPages,
    AsyncMarketSecuritiesPages,
    Security,
    market_securities_query,
    market_securities_pages,
    fetch_market_securities_page,
    args {
        engine: EngineName,
        market: MarketName
    },
    query {},
    market_securities_endpoint(engine, market),
    Securities,
    decode_securities_json_with_endpoint
);
paginated_endpoint!(
    SecuritiesPages,
    AsyncSecuritiesPages,
    Security,
    securities_query,
    securities_pages,
    fetch_securities_page,
    args {
        engine: EngineName,
        market: MarketName,
        board: BoardId
    },
    query {},
    securities_endpoint(engine, market, board),
    Securities,
    decode_securities_json_with_endpoint
);
paginated_endpoint!(
    MarketTradesPages,
    AsyncMarketTradesPages,
    Trade,
    market_trades_query,
    market_trades_pages,
    fetch_market_trades_page,
    args {
        engine: EngineName,
        market: MarketName
    },
    query {},
    market_trades_endpoint(engine, market),
    Trades,
    decode_trades_json_with_endpoint
);
paginated_endpoint!(
    TradesPages,
    AsyncTradesPages,
    Trade,
    trades_query,
    trades_pages,
    fetch_trades_page,
    args {
        engine: EngineName,
        market: MarketName,
        board: BoardId,
        security: SecId
    },
    query {},
    trades_endpoint(engine, market, board, security),
    Trades,
    decode_trades_json_with_endpoint
);
paginated_endpoint!(
    SecStatsPages,
    AsyncSecStatsPages,
    SecStat,
    secstats_query,
    secstats_pages,
    fetch_secstats_page,
    args {
        engine: EngineName,
        market: MarketName
    },
    query {},
    secstats_endpoint(engine, market),
    SecStats,
    decode_secstats_json_with_endpoint
);
paginated_endpoint!(
    CandlesPages, AsyncCandlesPages, Candle,
    candles_query, candles_pages, fetch_candles_page,
    args { engine: EngineName, market: MarketName, board: BoardId, security: SecId }, query { query: CandleQuery => append_candle_query_to_url }, candles_endpoint(engine, market, board, security), Candles, decode_candles_json_with_endpoint
);
#[cfg(feature = "news")]
paginated_endpoint!(
    SiteNewsPages,
    AsyncSiteNewsPages,
    SiteNews,
    sitenews_query,
    sitenews_pages,
    fetch_sitenews_page,
    args {},
    query {},
    SITENEWS_ENDPOINT,
    SiteNews,
    decode_sitenews_json_with_endpoint
);
#[cfg(feature = "news")]
paginated_endpoint!(
    EventsPages,
    AsyncEventsPages,
    Event,
    events_query,
    events_pages,
    fetch_events_page,
    args {},
    query {},
    EVENTS_ENDPOINT,
    Events,
    decode_events_json_with_endpoint
);
#[cfg(feature = "history")]
paginated_endpoint!(
    HistoryPages, AsyncHistoryPages, HistoryRecord,
    history_query_filtered, history_pages_filtered, fetch_history_page,
    args { engine: EngineName, market: MarketName, board: BoardId, security: SecId }, query { query: HistoryQuery => append_history_query_to_url }, history_endpoint(engine, market, board, security), History, decode_history_json_with_endpoint
);

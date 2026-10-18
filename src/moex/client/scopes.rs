use super::*;

macro_rules! owned_index_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $index_analytics_pages:ident) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) indexid: IndexId,
        }
        impl<'a> $name<'a> {
            /// Идентификатор индекса текущего контекста.
            pub fn indexid(&self) -> &IndexId {
                &self.indexid
            }

            /// Получить состав индекса (`analytics`) для текущего контекста.
            pub $($asyncness)? fn analytics(
                &self,
                page_request: PageRequest,
            ) -> Result<Vec<IndexAnalytics>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .index_analytics_query(&self.indexid, page_request)
                )
            }

            /// Создать ленивый пагинатор страниц `analytics` для текущего индекса.
            pub fn analytics_pages(&self, page_limit: NonZeroU32) -> $index_analytics_pages<'_> {
                self.client.index_analytics_pages(&self.indexid, page_limit)
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_index_scope!(
    OwnedIndexScope,
    BlockingMoexClient,
    blocking,
    [],
    IndexAnalyticsPages
);
#[cfg(feature = "async")]
owned_index_scope!(AsyncOwnedIndexScope, AsyncMoexClient, asynchronous, [async], AsyncIndexAnalyticsPages);

macro_rules! owned_engine_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $owned_market_scope:ident) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) engine: EngineName,
        }
        impl<'a> $name<'a> {
            /// Имя торгового движка текущего контекста.
            pub fn engine(&self) -> &EngineName {
                &self.engine
            }

            /// Получить доступные рынки (`markets`) для текущего движка.
            pub $($asyncness)? fn markets(&self) -> Result<Vec<Market>, MoexError> {
                await_result!($mode, self.client.markets(&self.engine))
            }

            /// Получить обороты (`turnovers`) для текущего движка.
            pub $($asyncness)? fn turnovers(&self) -> Result<Vec<Turnover>, MoexError> {
                await_result!($mode, self.client.engine_turnovers(&self.engine))
            }

            /// Зафиксировать рынок внутри текущего `engine`.
            pub fn market<M>(self, market: M) -> Result<$owned_market_scope<'a>, ParseMarketNameError>
            where
                M: TryInto<MarketName>,
                M::Error: Into<ParseMarketNameError>,
            {
                let market = market.try_into().map_err(Into::into)?;
                Ok($owned_market_scope {
                    client: self.client,
                    engine: self.engine,
                    market,
                })
            }

            /// Сокращение для часто используемого рынка `shares`.
            pub fn shares(self) -> Result<$owned_market_scope<'a>, ParseMarketNameError> {
                self.market("shares")
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_engine_scope!(
    OwnedEngineScope,
    BlockingMoexClient,
    blocking,
    [],
    OwnedMarketScope
);
#[cfg(feature = "async")]
owned_engine_scope!(AsyncOwnedEngineScope, AsyncMoexClient, asynchronous, [async], AsyncOwnedMarketScope);

macro_rules! owned_market_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $market_securities_pages:ident, $market_trades_pages:ident, $sec_stats_pages:ident, $owned_market_security_scope:ident, $owned_board_scope:ident) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) engine: EngineName,
            pub(super) market: MarketName,
        }
        impl<'a> $name<'a> {
            /// Имя торгового движка текущего контекста.
            pub fn engine(&self) -> &EngineName {
                &self.engine
            }

            /// Имя рынка текущего контекста.
            pub fn market(&self) -> &MarketName {
                &self.market
            }

            /// Получить режимы торгов (`boards`) для текущего рынка.
            pub $($asyncness)? fn boards(&self) -> Result<Vec<Board>, MoexError> {
                await_result!($mode, self.client.boards(&self.engine, &self.market))
            }

            /// Получить инструменты (`securities`) на уровне текущего рынка.
            pub $($asyncness)? fn securities(&self, page_request: PageRequest) -> Result<Vec<Security>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .market_securities_query(&self.engine, &self.market, page_request)
                )
            }

            /// Создать ленивый пагинатор страниц `securities` на уровне рынка.
            pub fn securities_pages(&self, page_limit: NonZeroU32) -> $market_securities_pages<'_> {
                self.client
                    .market_securities_pages(&self.engine, &self.market, page_limit)
            }

            /// Получить стакан на уровне рынка (`orderbook`) для текущего рынка.
            pub $($asyncness)? fn orderbook(&self) -> Result<Vec<OrderbookLevel>, MoexError> {
                await_result!(
                    $mode,
                    self.client.market_orderbook(&self.engine, &self.market)
                )
            }

            /// Получить сделки на уровне рынка (`trades`) для текущего рынка.
            pub $($asyncness)? fn trades(&self, page_request: PageRequest) -> Result<Vec<Trade>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .market_trades_query(&self.engine, &self.market, page_request)
                )
            }

            /// Создать ленивый пагинатор страниц `trades` на уровне рынка.
            pub fn trades_pages(&self, page_limit: NonZeroU32) -> $market_trades_pages<'_> {
                self.client
                    .market_trades_pages(&self.engine, &self.market, page_limit)
            }

            /// Получить `secstats` для текущего рынка.
            pub $($asyncness)? fn secstats(&self, page_request: PageRequest) -> Result<Vec<SecStat>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .secstats_query(&self.engine, &self.market, page_request)
                )
            }

            /// Создать ленивый пагинатор страниц `secstats`.
            pub fn secstats_pages(&self, page_limit: NonZeroU32) -> $sec_stats_pages<'_> {
                self.client
                    .secstats_pages(&self.engine, &self.market, page_limit)
            }

            /// Получить доступные границы свечей (`candleborders`) по инструменту.
            pub $($asyncness)? fn candle_borders(&self, security: &SecId) -> Result<Vec<CandleBorder>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .candle_borders(&self.engine, &self.market, security)
                )
            }

            /// Зафиксировать инструмент в рамках текущего `engine/market`.
            pub fn security<S>(
                self,
                security: S,
            ) -> Result<$owned_market_security_scope<'a>, ParseSecIdError>
            where
                S: TryInto<SecId>,
                S::Error: Into<ParseSecIdError>,
            {
                let security = security.try_into().map_err(Into::into)?;
                Ok($owned_market_security_scope {
                    client: self.client,
                    engine: self.engine,
                    market: self.market,
                    security,
                })
            }

            /// Зафиксировать `board` внутри текущего `engine/market`.
            pub fn board<B>(self, board: B) -> Result<$owned_board_scope<'a>, ParseBoardIdError>
            where
                B: TryInto<BoardId>,
                B::Error: Into<ParseBoardIdError>,
            {
                let board = board.try_into().map_err(Into::into)?;
                Ok($owned_board_scope {
                    client: self.client,
                    engine: self.engine,
                    market: self.market,
                    board,
                })
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_market_scope!(
    OwnedMarketScope,
    BlockingMoexClient,
    blocking,
    [],
    MarketSecuritiesPages,
    MarketTradesPages,
    SecStatsPages,
    OwnedMarketSecurityScope,
    OwnedBoardScope
);
#[cfg(feature = "async")]
owned_market_scope!(AsyncOwnedMarketScope, AsyncMoexClient, asynchronous, [async], AsyncMarketSecuritiesPages, AsyncMarketTradesPages, AsyncSecStatsPages, AsyncOwnedMarketSecurityScope, AsyncOwnedBoardScope);

macro_rules! owned_market_security_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?]) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) engine: EngineName,
            pub(super) market: MarketName,
            pub(super) security: SecId,
        }
        impl<'a> $name<'a> {
            /// Имя торгового движка текущего контекста.
            pub fn engine(&self) -> &EngineName {
                &self.engine
            }

            /// Имя рынка текущего контекста.
            pub fn market(&self) -> &MarketName {
                &self.market
            }

            /// Идентификатор инструмента текущего контекста.
            pub fn security(&self) -> &SecId {
                &self.security
            }

            /// Получить карточку текущего инструмента на уровне рынка.
            pub $($asyncness)? fn info(&self) -> Result<Option<Security>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .market_security_info(&self.engine, &self.market, &self.security)
                )
            }

            /// Получить доступные границы свечей (`candleborders`) по текущему инструменту.
            pub $($asyncness)? fn candle_borders(&self) -> Result<Vec<CandleBorder>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .candle_borders(&self.engine, &self.market, &self.security)
                )
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_market_security_scope!(OwnedMarketSecurityScope, BlockingMoexClient, blocking, []);
#[cfg(feature = "async")]
owned_market_security_scope!(AsyncOwnedMarketSecurityScope, AsyncMoexClient, asynchronous, [async]);

macro_rules! owned_board_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $securities_pages:ident, $owned_security_scope:ident) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) engine: EngineName,
            pub(super) market: MarketName,
            pub(super) board: BoardId,
        }
        impl<'a> $name<'a> {
            /// Имя торгового движка текущего контекста.
            pub fn engine(&self) -> &EngineName {
                &self.engine
            }

            /// Имя рынка текущего контекста.
            pub fn market(&self) -> &MarketName {
                &self.market
            }

            /// Идентификатор режима торгов текущего контекста.
            pub fn board(&self) -> &BoardId {
                &self.board
            }

            /// Получить инструменты (`securities`) для текущего контекста.
            pub $($asyncness)? fn securities(&self, page_request: PageRequest) -> Result<Vec<Security>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .securities_query(&self.engine, &self.market, &self.board, page_request)
                )
            }

            /// Создать ленивый пагинатор страниц `securities` для текущего контекста.
            pub fn securities_pages(&self, page_limit: NonZeroU32) -> $securities_pages<'_> {
                self.client
                    .securities_pages(&self.engine, &self.market, &self.board, page_limit)
            }

            /// Получить снимки инструментов (`LOTSIZE` и `LAST`) для текущего контекста.
            pub $($asyncness)? fn snapshots(&self) -> Result<Vec<SecuritySnapshot>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .board_snapshots(&self.engine, &self.market, &self.board)
                )
            }

            /// Зафиксировать инструмент в рамках текущего `engine/market/board`.
            pub fn security<S>(self, security: S) -> Result<$owned_security_scope<'a>, ParseSecIdError>
            where
                S: TryInto<SecId>,
                S::Error: Into<ParseSecIdError>,
            {
                let security = security.try_into().map_err(Into::into)?;
                Ok($owned_security_scope {
                    client: self.client,
                    engine: self.engine,
                    market: self.market,
                    board: self.board,
                    security,
                })
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_board_scope!(
    OwnedBoardScope,
    BlockingMoexClient,
    blocking,
    [],
    SecuritiesPages,
    OwnedSecurityScope
);
#[cfg(feature = "async")]
owned_board_scope!(AsyncOwnedBoardScope, AsyncMoexClient, asynchronous, [async], AsyncSecuritiesPages, AsyncOwnedSecurityScope);

macro_rules! owned_security_resource_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?]) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) security: SecId,
        }
        impl<'a> $name<'a> {
            /// Идентификатор инструмента текущего контекста.
            pub fn secid(&self) -> &SecId {
                &self.security
            }

            /// Получить карточку текущего инструмента.
            pub $($asyncness)? fn info(&self) -> Result<Option<Security>, MoexError> {
                await_result!($mode, self.client.security_info(&self.security))
            }

            /// Получить режимы торгов (`boards`) для текущего инструмента.
            pub $($asyncness)? fn boards(&self) -> Result<Vec<SecurityBoard>, MoexError> {
                await_result!($mode, self.client.security_boards(&self.security))
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_security_resource_scope!(OwnedSecurityResourceScope, BlockingMoexClient, blocking, []);
#[cfg(feature = "async")]
owned_security_resource_scope!(AsyncOwnedSecurityResourceScope, AsyncMoexClient, asynchronous, [async]);

macro_rules! owned_security_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $trades_pages:ident, $history_pages:ident, $candles_pages:ident) => {
        /// Владеющий контекст ISS для fluent-запросов.
        #[derive(Clone)]
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            pub(super) engine: EngineName,
            pub(super) market: MarketName,
            pub(super) board: BoardId,
            pub(super) security: SecId,
        }
        impl<'a> $name<'a> {
            /// Идентификатор инструмента текущего контекста.
            pub fn security(&self) -> &SecId {
                &self.security
            }

            /// Получить стакан (`orderbook`) по текущему инструменту.
            pub $($asyncness)? fn orderbook(&self) -> Result<Vec<OrderbookLevel>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .orderbook(&self.engine, &self.market, &self.board, &self.security)
                )
            }

            #[cfg(feature = "history")]
            /// Получить диапазон доступных исторических дат по текущему инструменту.
            pub $($asyncness)? fn history_dates(&self) -> Result<Option<HistoryDates>, MoexError> {
                await_result!(
                    $mode,
                    self.client
                        .history_dates(&self.engine, &self.market, &self.board, &self.security)
                )
            }

            #[cfg(feature = "history")]
            /// Получить исторические данные (`history`) по текущему инструменту.
            pub $($asyncness)? fn history(
                &self,
                page_request: PageRequest,
            ) -> Result<Vec<HistoryRecord>, MoexError> {
                await_result!(
                    $mode,
                    self.client.history_query(
                        &self.engine,
                        &self.market,
                        &self.board,
                        &self.security,
                        page_request,
                    )
                )
            }

            #[cfg(feature = "history")]
            /// Получить отфильтрованные по датам исторические данные по текущему инструменту.
            pub $($asyncness)? fn history_filtered(
                &self,
                query: HistoryQuery,
                page_request: PageRequest,
            ) -> Result<Vec<HistoryRecord>, MoexError> {
                await_result!(
                    $mode,
                    self.client.history_query_filtered(
                        &self.engine,
                        &self.market,
                        &self.board,
                        &self.security,
                        query,
                        page_request,
                    )
                )
            }

            #[cfg(feature = "history")]
            /// Создать ленивый пагинатор страниц `history` по текущему инструменту.
            pub fn history_pages(&self, page_limit: NonZeroU32) -> $history_pages<'_> {
                self.client.history_pages(
                    &self.engine,
                    &self.market,
                    &self.board,
                    &self.security,
                    page_limit,
                )
            }

            #[cfg(feature = "history")]
            /// Создать пагинатор отфильтрованных страниц `history` по текущему инструменту.
            pub fn history_pages_filtered(
                &self,
                query: HistoryQuery,
                page_limit: NonZeroU32,
            ) -> $history_pages<'_> {
                self.client.history_pages_filtered(
                    &self.engine,
                    &self.market,
                    &self.board,
                    &self.security,
                    query,
                    page_limit,
                )
            }

            /// Получить сделки (`trades`) по текущему инструменту.
            pub $($asyncness)? fn trades(&self, page_request: PageRequest) -> Result<Vec<Trade>, MoexError> {
                await_result!(
                    $mode,
                    self.client.trades_query(
                        &self.engine,
                        &self.market,
                        &self.board,
                        &self.security,
                        page_request,
                    )
                )
            }

            /// Создать ленивый пагинатор страниц `trades` по текущему инструменту.
            pub fn trades_pages(&self, page_limit: NonZeroU32) -> $trades_pages<'_> {
                self.client.trades_pages(
                    &self.engine,
                    &self.market,
                    &self.board,
                    &self.security,
                    page_limit,
                )
            }

            /// Получить свечи (`candles`) по текущему инструменту.
            pub $($asyncness)? fn candles(
                &self,
                query: CandleQuery,
                page_request: PageRequest,
            ) -> Result<Vec<Candle>, MoexError> {
                await_result!(
                    $mode,
                    self.client.candles_query(
                        &self.engine,
                        &self.market,
                        &self.board,
                        &self.security,
                        query,
                        page_request,
                    )
                )
            }

            /// Создать ленивый пагинатор страниц `candles` по текущему инструменту.
            pub fn candles_pages(
                &self,
                query: CandleQuery,
                page_limit: NonZeroU32,
            ) -> $candles_pages<'_> {
                self.client.candles_pages(
                    &self.engine,
                    &self.market,
                    &self.board,
                    &self.security,
                    query,
                    page_limit,
                )
            }
        }
    };
}

#[cfg(feature = "blocking")]
owned_security_scope!(
    OwnedSecurityScope,
    BlockingMoexClient,
    blocking,
    [],
    TradesPages,
    HistoryPages,
    CandlesPages
);
#[cfg(feature = "async")]
owned_security_scope!(AsyncOwnedSecurityScope, AsyncMoexClient, asynchronous, [async], AsyncTradesPages, AsyncHistoryPages, AsyncCandlesPages);

#[cfg(feature = "blocking")]
impl OwnedSecurityScope<'_> {
    /// Получить доступные границы свечей (`candleborders`) по текущему инструменту.
    pub fn candle_borders(&self) -> Result<Vec<CandleBorder>, MoexError> {
        self.client
            .candle_borders(&self.engine, &self.market, &self.security)
    }
}

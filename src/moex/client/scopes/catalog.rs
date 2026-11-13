use super::*;

macro_rules! owned_engine_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $owned_market_scope:ident) => {
        owned_scope!($name, $client, {
            /// Имя торгового движка текущего контекста.
            engine: EngineName => engine,
        });
        impl<'a> $name<'a> {
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
        owned_scope!($name, $client, {
            /// Имя торгового движка текущего контекста.
            engine: EngineName => engine,
            /// Имя рынка текущего контекста.
            market: MarketName => market,
        });
        impl<'a> $name<'a> {
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

macro_rules! owned_board_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $securities_pages:ident, $owned_security_scope:ident) => {
        owned_scope!($name, $client, {
            /// Имя торгового движка текущего контекста.
            engine: EngineName => engine,
            /// Имя рынка текущего контекста.
            market: MarketName => market,
            /// Идентификатор режима торгов текущего контекста.
            board: BoardId => board,
        });
        impl<'a> $name<'a> {
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

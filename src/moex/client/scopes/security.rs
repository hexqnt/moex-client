use super::*;

macro_rules! owned_market_security_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?]) => {
        owned_scope!($name, $client, {
            /// Имя торгового движка текущего контекста.
            engine: EngineName => engine,
            /// Имя рынка текущего контекста.
            market: MarketName => market,
            /// Идентификатор инструмента текущего контекста.
            security: SecId => security,
        });
        impl<'a> $name<'a> {
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

macro_rules! owned_security_resource_scope {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?]) => {
        owned_scope!($name, $client, {
            /// Идентификатор инструмента текущего контекста.
            security: SecId => secid,
        });
        impl<'a> $name<'a> {
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
        owned_scope!($name, $client, {
            engine: EngineName,
            market: MarketName,
            board: BoardId,
            /// Идентификатор инструмента текущего контекста.
            security: SecId => security,
        });
        impl<'a> $name<'a> {
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

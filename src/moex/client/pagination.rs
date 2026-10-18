use super::*;

macro_rules! pages {
    ($name:ident, $client:ty, $mode:ident, [$($asyncness:tt)?], $item:ty, $key:ty,
     args { $($arg:ident: $ty:ty),* }, query { $($query:ident: $query_ty:ty)? },
     $fetch:ident, $first_key:expr) => {
        /// Ленивый пагинатор по страницам ISS.
        pub struct $name<'a> {
            pub(super) client: &'a $client,
            $(pub(super) $arg: &'a $ty,)*
            $(pub(super) $query: $query_ty,)?
            pub(super) pagination: PaginationTracker<$key>,
        }

        impl $name<'_> {
            /// Получить следующую страницу ISS.
            pub $($asyncness)? fn next_page(&mut self) -> Result<Option<Vec<$item>>, MoexError> {
                let Some(paging) = self.pagination.next_page_request() else {
                    return Ok(None);
                };
                let page = await_result!($mode, self.client.$fetch($(self.$arg,)* $(self.$query,)? paging))?;
                self.pagination.accept_page(page, $first_key)
            }

            /// Собрать все страницы в один `Vec`.
            pub $($asyncness)? fn try_collect(mut self) -> Result<Vec<$item>, MoexError> {
                let mut items = Vec::new();
                while let Some(page) = await_result!($mode, self.next_page())? {
                    items.extend(page);
                }
                Ok(items)
            }

            /// Алиас для [`Self::try_collect`].
            pub $($asyncness)? fn all(self) -> Result<Vec<$item>, MoexError> {
                await_result!($mode, self.try_collect())
            }
        }
    };
}

macro_rules! paginators {
    ($blocking:ident, $asynchronous:ident, $item:ty, $key:ty,
     args { $($arg:ident: $ty:ty),* }, query { $($query:ident: $query_ty:ty)? },
     $fetch:ident, $first_key:expr) => {
        #[cfg(feature = "blocking")]
        pages!($blocking, BlockingMoexClient, blocking, [], $item, $key,
            args { $($arg: $ty),* }, query { $($query: $query_ty)? }, $fetch, $first_key);
        #[cfg(feature = "async")]
        pages!($asynchronous, AsyncMoexClient, asynchronous, [async], $item, $key,
            args { $($arg: $ty),* }, query { $($query: $query_ty)? }, $fetch, $first_key);
    };
}

paginators!(
    IndexAnalyticsPages,
    AsyncIndexAnalyticsPages,
    IndexAnalytics,
    (chrono::NaiveDate, SecId),
    args { indexid: IndexId },
    query {},
    fetch_index_analytics_page,
    |item: &IndexAnalytics| (item.trade_session_date(), item.secid().clone())
);
paginators!(
    SecuritiesPages,
    AsyncSecuritiesPages,
    Security,
    SecId,
    args {
        engine: EngineName,
        market: MarketName,
        board: BoardId
    },
    query {},
    fetch_securities_page,
    |item: &Security| item.secid().clone()
);
paginators!(
    GlobalSecuritiesPages,
    AsyncGlobalSecuritiesPages,
    Security,
    SecId,
    args {},
    query {},
    fetch_global_securities_page,
    |item: &Security| item.secid().clone()
);
paginators!(
    MarketSecuritiesPages,
    AsyncMarketSecuritiesPages,
    Security,
    SecId,
    args {
        engine: EngineName,
        market: MarketName
    },
    query {},
    fetch_market_securities_page,
    |item: &Security| item.secid().clone()
);
paginators!(
    MarketTradesPages,
    AsyncMarketTradesPages,
    Trade,
    u64,
    args {
        engine: EngineName,
        market: MarketName
    },
    query {},
    fetch_market_trades_page,
    Trade::tradeno
);
paginators!(
    TradesPages,
    AsyncTradesPages,
    Trade,
    u64,
    args {
        engine: EngineName,
        market: MarketName,
        board: BoardId,
        security: SecId
    },
    query {},
    fetch_trades_page,
    Trade::tradeno
);
paginators!(
    SecStatsPages,
    AsyncSecStatsPages,
    SecStat,
    (SecId, BoardId),
    args {
        engine: EngineName,
        market: MarketName
    },
    query {},
    fetch_secstats_page,
    |item: &SecStat| (item.secid().clone(), item.boardid().clone())
);
paginators!(
    CandlesPages,
    AsyncCandlesPages,
    Candle,
    chrono::NaiveDateTime,
    args {
        engine: EngineName,
        market: MarketName,
        board: BoardId,
        security: SecId
    },
    query { query: CandleQuery },
    fetch_candles_page,
    Candle::begin
);
#[cfg(feature = "history")]
paginators!(
    HistoryPages,
    AsyncHistoryPages,
    HistoryRecord,
    chrono::NaiveDate,
    args {
        engine: EngineName,
        market: MarketName,
        board: BoardId,
        security: SecId
    },
    query {
        query: HistoryQuery
    },
    fetch_history_page,
    HistoryRecord::tradedate
);
#[cfg(feature = "news")]
paginators!(
    SiteNewsPages,
    AsyncSiteNewsPages,
    SiteNews,
    u64,
    args {},
    query {},
    fetch_sitenews_page,
    SiteNews::id
);
#[cfg(feature = "news")]
paginators!(
    EventsPages,
    AsyncEventsPages,
    Event,
    u64,
    args {},
    query {},
    fetch_events_page,
    Event::id
);

enum PaginationAdvance {
    YieldPage,
    EndOfPages,
}

pub(super) struct PaginationTracker<K> {
    endpoint: Box<str>,
    page_limit: NonZeroU32,
    start: u32,
    first_key_on_previous_page: Option<K>,
    finished: bool,
}

impl<K> PaginationTracker<K> {
    pub(super) fn new(endpoint: impl Into<String>, page_limit: NonZeroU32) -> Self {
        Self {
            endpoint: endpoint.into().into_boxed_str(),
            page_limit,
            start: 0,
            first_key_on_previous_page: None,
            finished: false,
        }
    }

    fn next_page_request(&self) -> Option<Pagination> {
        if self.finished {
            return None;
        }
        Some(Pagination {
            start: Some(self.start),
            limit: Some(self.page_limit),
        })
    }
}

impl<K> PaginationTracker<K>
where
    K: Eq,
{
    fn advance(
        &mut self,
        page_len: usize,
        first_key_on_page: Option<K>,
    ) -> Result<PaginationAdvance, MoexError> {
        let page_limit = self.page_limit.get();

        if page_len == 0 {
            self.finished = true;
            return Ok(PaginationAdvance::EndOfPages);
        }

        if let (Some(prev), Some(current)) = (&self.first_key_on_previous_page, &first_key_on_page)
            && prev == current
        {
            return Err(MoexError::PaginationStuck {
                endpoint: self.endpoint.clone(),
                start: self.start,
                limit: page_limit,
            });
        }

        self.first_key_on_previous_page = first_key_on_page;

        if (page_len as u128) < u128::from(page_limit) {
            self.finished = true;
            return Ok(PaginationAdvance::YieldPage);
        }

        self.start =
            self.start
                .checked_add(page_limit)
                .ok_or_else(|| MoexError::PaginationOverflow {
                    endpoint: self.endpoint.clone(),
                    start: self.start,
                    limit: page_limit,
                })?;

        Ok(PaginationAdvance::YieldPage)
    }
}

impl<K: Eq> PaginationTracker<K> {
    fn accept_page<T>(
        &mut self,
        page: Vec<T>,
        first_key_of: impl FnOnce(&T) -> K,
    ) -> Result<Option<Vec<T>>, MoexError> {
        let first_key = page.first().map(first_key_of);
        match self.advance(page.len(), first_key)? {
            PaginationAdvance::YieldPage => Ok(Some(page)),
            PaginationAdvance::EndOfPages => Ok(None),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

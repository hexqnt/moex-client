use std::num::NonZeroU32;
#[cfg(any(feature = "blocking", feature = "async"))]
use std::sync::Mutex;
use std::time::Duration;

#[cfg(feature = "blocking")]
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::{Url, header::HeaderMap};

use crate::models::{
    Board, BoardId, Candle, CandleBorder, CandleQuery, Engine, EngineName, Index, IndexAnalytics,
    IndexId, Market, MarketName, OrderbookLevel, PageRequest, Pagination, ParseBoardIdError,
    ParseEngineNameError, ParseIndexError, ParseMarketNameError, ParseSecIdError, SecId, SecStat,
    Security, SecurityBoard, SecuritySnapshot, Trade, Turnover,
};
#[cfg(feature = "news")]
use crate::models::{Event, SiteNews};
#[cfg(feature = "history")]
use crate::models::{HistoryDates, HistoryQuery, HistoryRecord};

use super::constants::*;
use super::payload::{
    decode_board_security_snapshots_json_with_endpoint, decode_boards_json_with_endpoint,
    decode_candle_borders_json_with_endpoint, decode_candles_json_with_endpoint,
    decode_engines_json_with_endpoint, decode_index_analytics_json_with_endpoint,
    decode_indexes_json_with_endpoint, decode_markets_json_with_endpoint,
    decode_orderbook_json_with_endpoint, decode_raw_table_rows_json_with_endpoint,
    decode_secstats_json_with_endpoint, decode_securities_json_with_endpoint,
    decode_security_boards_json_with_endpoint, decode_trades_json_with_endpoint,
    decode_turnovers_json_with_endpoint,
};
#[cfg(feature = "news")]
use super::payload::{decode_events_json_with_endpoint, decode_sitenews_json_with_endpoint};
#[cfg(feature = "history")]
use super::payload::{decode_history_dates_json_with_endpoint, decode_history_json_with_endpoint};
use super::{IssEndpoint, IssRequestOptions, IssToggle, MoexError, RawIssResponse};
#[cfg(any(feature = "blocking", feature = "async"))]
use super::{RateLimit, RateLimiter};

#[cfg(feature = "async")]
type AsyncSleepFuture = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static>>;

#[cfg(feature = "async")]
type AsyncRateLimitSleep = std::sync::Arc<dyn Fn(Duration) -> AsyncSleepFuture + Send + Sync>;

macro_rules! await_result {
    (blocking, $expression:expr) => {
        $expression
    };
    (asynchronous, $expression:expr) => {
        $expression.await
    };
}

mod builders;
mod endpoints;
mod pagination;
mod raw;
mod request;
mod scopes;
mod transport;

use pagination::PaginationTracker;
use request::*;

pub use builders::*;
pub use endpoints::*;
pub use raw::*;
pub use scopes::*;
pub use transport::*;

pub(super) use request::{
    append_candle_query_to_url, append_pagination_to_url, optional_single_security,
};
#[cfg(feature = "history")]
pub(super) use request::{append_history_query_to_url, optional_single_history_dates};
#[cfg(all(test, feature = "blocking"))]
pub(super) use request::{looks_like_json_payload, normalize_raw_endpoint_path, truncate_prefix};

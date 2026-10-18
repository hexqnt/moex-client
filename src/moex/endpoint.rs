//! Типизированные пути и таблицы ISS.

use crate::models::{BoardId, EngineName, IndexId, MarketName, SecId};

use super::constants;

/// Типизированное описание ISS endpoint-а для raw-запросов.
///
/// Позволяет строить raw-запросы без ручной сборки строк пути.
#[derive(Debug, Clone, Copy)]
pub enum IssEndpoint<'a> {
    /// `/iss/statistics/engines/stock/markets/index/analytics.json` (`indices`).
    Indexes,
    /// `/iss/statistics/engines/stock/markets/index/analytics/{indexid}.json` (`analytics`).
    IndexAnalytics { indexid: &'a IndexId },
    /// `/iss/turnovers.json` (`turnovers`).
    Turnovers,
    /// `/iss/engines/{engine}/turnovers.json` (`turnovers`).
    EngineTurnovers { engine: &'a EngineName },
    /// `/iss/engines.json` (`engines`).
    Engines,
    /// `/iss/engines/{engine}/markets.json` (`markets`).
    Markets { engine: &'a EngineName },
    /// `/iss/engines/{engine}/markets/{market}/boards.json` (`boards`).
    Boards {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/securities.json` (`securities`).
    GlobalSecurities,
    /// `/iss/securities/{secid}.json` (`securities`).
    SecurityInfo { security: &'a SecId },
    /// `/iss/securities/{secid}.json` (`boards`).
    SecurityBoards { security: &'a SecId },
    /// `/iss/engines/{engine}/markets/{market}/securities.json` (`securities`).
    MarketSecurities {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/securities/{secid}.json` (`securities`).
    MarketSecurityInfo {
        engine: &'a EngineName,
        market: &'a MarketName,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/orderbook.json` (`orderbook`).
    MarketOrderbook {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/trades.json` (`trades`).
    MarketTrades {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/secstats.json` (`secstats`).
    SecStats {
        engine: &'a EngineName,
        market: &'a MarketName,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities.json` (`securities`).
    Securities {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities.json` (`securities,marketdata`).
    BoardSecuritySnapshots {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/orderbook.json` (`orderbook`).
    Orderbook {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/trades.json` (`trades`).
    Trades {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/candles.json` (`candles`).
    Candles {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/engines/{engine}/markets/{market}/securities/{secid}/candleborders.json` (`borders`).
    CandleBorders {
        engine: &'a EngineName,
        market: &'a MarketName,
        security: &'a SecId,
    },
    /// `/iss/sitenews.json` (`sitenews`).
    #[cfg(feature = "news")]
    SiteNews,
    /// `/iss/events.json` (`events`).
    #[cfg(feature = "news")]
    Events,
    /// `/iss/history/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/dates.json` (`dates`).
    #[cfg(feature = "history")]
    HistoryDates {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
    /// `/iss/history/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}.json` (`history`).
    #[cfg(feature = "history")]
    History {
        engine: &'a EngineName,
        market: &'a MarketName,
        board: &'a BoardId,
        security: &'a SecId,
    },
}

impl IssEndpoint<'_> {
    /// Построить относительный endpoint-path (`*.json`) для raw-запроса.
    pub fn path(self) -> String {
        match self {
            Self::Indexes => constants::INDEXES_ENDPOINT.to_owned(),
            Self::IndexAnalytics { indexid } => constants::index_analytics_endpoint(indexid),
            Self::Turnovers => constants::TURNOVERS_ENDPOINT.to_owned(),
            Self::EngineTurnovers { engine } => constants::engine_turnovers_endpoint(engine),
            Self::Engines => constants::ENGINES_ENDPOINT.to_owned(),
            Self::Markets { engine } => constants::markets_endpoint(engine),
            Self::Boards { engine, market } => constants::boards_endpoint(engine, market),
            Self::GlobalSecurities => constants::GLOBAL_SECURITIES_ENDPOINT.to_owned(),
            Self::SecurityInfo { security } | Self::SecurityBoards { security } => {
                constants::security_endpoint(security)
            }
            Self::MarketSecurities { engine, market } => {
                constants::market_securities_endpoint(engine, market)
            }
            Self::MarketSecurityInfo {
                engine,
                market,
                security,
            } => constants::market_security_endpoint(engine, market, security),
            Self::MarketOrderbook { engine, market } => {
                constants::market_orderbook_endpoint(engine, market)
            }
            Self::MarketTrades { engine, market } => {
                constants::market_trades_endpoint(engine, market)
            }
            Self::SecStats { engine, market } => constants::secstats_endpoint(engine, market),
            Self::Securities {
                engine,
                market,
                board,
            }
            | Self::BoardSecuritySnapshots {
                engine,
                market,
                board,
            } => constants::securities_endpoint(engine, market, board),
            Self::Orderbook {
                engine,
                market,
                board,
                security,
            } => constants::orderbook_endpoint(engine, market, board, security),
            Self::Trades {
                engine,
                market,
                board,
                security,
            } => constants::trades_endpoint(engine, market, board, security),
            Self::Candles {
                engine,
                market,
                board,
                security,
            } => constants::candles_endpoint(engine, market, board, security),
            Self::CandleBorders {
                engine,
                market,
                security,
            } => constants::candleborders_endpoint(engine, market, security),
            #[cfg(feature = "news")]
            Self::SiteNews => constants::SITENEWS_ENDPOINT.to_owned(),
            #[cfg(feature = "news")]
            Self::Events => constants::EVENTS_ENDPOINT.to_owned(),
            #[cfg(feature = "history")]
            Self::HistoryDates {
                engine,
                market,
                board,
                security,
            } => constants::history_dates_endpoint(engine, market, board, security),
            #[cfg(feature = "history")]
            Self::History {
                engine,
                market,
                board,
                security,
            } => constants::history_endpoint(engine, market, board, security),
        }
    }

    /// Таблица по умолчанию для `iss.only`, если endpoint описывает единственную цель выборки.
    pub fn default_table(self) -> Option<&'static str> {
        match self {
            Self::Indexes => Some("indices"),
            Self::IndexAnalytics { .. } => Some("analytics"),
            Self::Turnovers | Self::EngineTurnovers { .. } => Some("turnovers"),
            Self::Engines => Some("engines"),
            Self::Markets { .. } => Some("markets"),
            Self::Boards { .. } | Self::SecurityBoards { .. } => Some("boards"),
            Self::GlobalSecurities
            | Self::SecurityInfo { .. }
            | Self::MarketSecurities { .. }
            | Self::MarketSecurityInfo { .. }
            | Self::Securities { .. } => Some("securities"),
            Self::BoardSecuritySnapshots { .. } => Some("securities,marketdata"),
            Self::Orderbook { .. } | Self::MarketOrderbook { .. } => Some("orderbook"),
            Self::Trades { .. } | Self::MarketTrades { .. } => Some("trades"),
            Self::Candles { .. } => Some("candles"),
            Self::CandleBorders { .. } => Some("borders"),
            Self::SecStats { .. } => Some("secstats"),
            #[cfg(feature = "news")]
            Self::SiteNews => Some("sitenews"),
            #[cfg(feature = "news")]
            Self::Events => Some("events"),
            #[cfg(feature = "history")]
            Self::HistoryDates { .. } => Some("dates"),
            #[cfg(feature = "history")]
            Self::History { .. } => Some("history"),
        }
    }
}

#[cfg(test)]
mod tests;

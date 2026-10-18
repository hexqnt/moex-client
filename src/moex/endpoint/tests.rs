use super::IssEndpoint::{self, *};
use crate::models::{BoardId, EngineName, IndexId, MarketName, SecId};

#[test]
fn endpoints_preserve_paths_and_default_tables() {
    let engine = &EngineName::try_from("stock").unwrap();
    let market = &MarketName::try_from("shares").unwrap();
    let board = &BoardId::try_from("TQBR").unwrap();
    let security = &SecId::try_from("SBER").unwrap();
    let indexid = &IndexId::try_from("IMOEX").unwrap();
    let cases: &[(IssEndpoint<'_>, &str, &str)] = &[
        (
            Indexes,
            "statistics/engines/stock/markets/index/analytics.json",
            "indices",
        ),
        (
            IndexAnalytics { indexid },
            "statistics/engines/stock/markets/index/analytics/IMOEX.json",
            "analytics",
        ),
        (Turnovers, "turnovers.json", "turnovers"),
        (
            EngineTurnovers { engine },
            "engines/stock/turnovers.json",
            "turnovers",
        ),
        (Engines, "engines.json", "engines"),
        (Markets { engine }, "engines/stock/markets.json", "markets"),
        (
            Boards { engine, market },
            "engines/stock/markets/shares/boards.json",
            "boards",
        ),
        (GlobalSecurities, "securities.json", "securities"),
        (
            SecurityInfo { security },
            "securities/SBER.json",
            "securities",
        ),
        (
            SecurityBoards { security },
            "securities/SBER.json",
            "boards",
        ),
        (
            MarketSecurities { engine, market },
            "engines/stock/markets/shares/securities.json",
            "securities",
        ),
        (
            MarketSecurityInfo {
                engine,
                market,
                security,
            },
            "engines/stock/markets/shares/securities/SBER.json",
            "securities",
        ),
        (
            MarketOrderbook { engine, market },
            "engines/stock/markets/shares/orderbook.json",
            "orderbook",
        ),
        (
            MarketTrades { engine, market },
            "engines/stock/markets/shares/trades.json",
            "trades",
        ),
        (
            SecStats { engine, market },
            "engines/stock/markets/shares/secstats.json",
            "secstats",
        ),
        (
            Securities {
                engine,
                market,
                board,
            },
            "engines/stock/markets/shares/boards/TQBR/securities.json",
            "securities",
        ),
        (
            BoardSecuritySnapshots {
                engine,
                market,
                board,
            },
            "engines/stock/markets/shares/boards/TQBR/securities.json",
            "securities,marketdata",
        ),
        (
            Orderbook {
                engine,
                market,
                board,
                security,
            },
            "engines/stock/markets/shares/boards/TQBR/securities/SBER/orderbook.json",
            "orderbook",
        ),
        (
            Trades {
                engine,
                market,
                board,
                security,
            },
            "engines/stock/markets/shares/boards/TQBR/securities/SBER/trades.json",
            "trades",
        ),
        (
            Candles {
                engine,
                market,
                board,
                security,
            },
            "engines/stock/markets/shares/boards/TQBR/securities/SBER/candles.json",
            "candles",
        ),
        (
            CandleBorders {
                engine,
                market,
                security,
            },
            "engines/stock/markets/shares/securities/SBER/candleborders.json",
            "borders",
        ),
        #[cfg(feature = "news")]
        (SiteNews, "sitenews.json", "sitenews"),
        #[cfg(feature = "news")]
        (Events, "events.json", "events"),
        #[cfg(feature = "history")]
        (
            HistoryDates {
                engine,
                market,
                board,
                security,
            },
            "history/engines/stock/markets/shares/boards/TQBR/securities/SBER/dates.json",
            "dates",
        ),
        #[cfg(feature = "history")]
        (
            History {
                engine,
                market,
                board,
                security,
            },
            "history/engines/stock/markets/shares/boards/TQBR/securities/SBER.json",
            "history",
        ),
    ];
    for &(endpoint, path, table) in cases {
        assert_eq!(endpoint.path(), path, "{endpoint:?}");
        assert_eq!(endpoint.default_table(), Some(table), "{endpoint:?}");
    }
}

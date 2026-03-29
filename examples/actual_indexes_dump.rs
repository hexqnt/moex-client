use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::time::Duration;

use moex_client::blocking::Client;
use moex_client::models::{IndexAnalytics, SecId, SecuritySnapshot};
use moex_client::prelude::*;
use moex_client::{MoexError, RateLimit, RetryPolicy, decode, with_retry};
use thiserror::Error;

const REQUEST_TIMEOUT_SECS: u64 = 5;
const REQUEST_RETRIES: u32 = 3;
const RETRY_DELAY_MILLIS: u64 = 400;
const RATE_LIMIT_MILLIS: u64 = 50;
const INDEX_ANALYTICS_PAGE_LIMIT: u32 = 5_000;
const BOARD_SNAPSHOTS_PAGE_LIMIT: u32 = 100;

#[derive(Debug, Error)]
enum ExampleError {
    #[error(transparent)]
    Moex(#[from] MoexError),
}

#[derive(Debug, Clone)]
struct IndexDump {
    index_id: Box<str>,
    short_name: Box<str>,
    components: Vec<IndexAnalytics>,
}

#[derive(Debug, Clone)]
struct ResolvedSnapshot {
    market: Box<str>,
    board: Box<str>,
    lot_size: Option<u32>,
    last: Option<f64>,
}

fn main() -> Result<(), ExampleError> {
    let moex_client = Client::builder()
        .user_agent_from_crate()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .rate_limit(RateLimit::every(Duration::from_millis(RATE_LIMIT_MILLIS)))
        .metadata(false)
        .build()?;

    println!("loading active indexes...");
    let index_dumps = load_actual_index_dumps(&moex_client)?;
    println!("active indexes loaded: {}", index_dumps.len());

    let wanted_secids = collect_unique_secids(&index_dumps);
    println!(
        "resolving snapshots for {} unique securities...",
        wanted_secids.len()
    );
    let (snapshots, missing_mapping) = load_security_snapshots(&moex_client, &wanted_secids)?;
    println!(
        "resolved snapshots: {}, missing: {}",
        snapshots.len(),
        missing_mapping.len()
    );

    print_dump(&index_dumps, &snapshots, &missing_mapping);
    Ok(())
}

fn load_actual_index_dumps(moex_client: &Client) -> Result<Vec<IndexDump>, ExampleError> {
    let indexes = with_retry(retry_policy(), || moex_client.indexes())?.into_actual_by_till();
    let page_limit = NonZeroU32::new(INDEX_ANALYTICS_PAGE_LIMIT)
        .expect("INDEX_ANALYTICS_PAGE_LIMIT constant must be greater than zero");

    indexes
        .into_iter()
        .map(|index| {
            let components = with_retry(retry_policy(), || {
                moex_client
                    .index(index.id().clone())
                    .expect("index id from payload must be valid")
                    .analytics_pages(page_limit)
                    .all()
            })?
            .into_actual_by_session()
            .into_sorted_by_weight_desc();
            Ok(IndexDump {
                index_id: index.id().as_str().to_owned().into_boxed_str(),
                short_name: index.short_name().to_owned().into_boxed_str(),
                components,
            })
        })
        .collect()
}

fn collect_unique_secids(index_dumps: &[IndexDump]) -> HashSet<SecId> {
    index_dumps
        .iter()
        .flat_map(|index| index.components.iter().map(|row| row.secid().clone()))
        .collect()
}

fn load_security_snapshots(
    moex_client: &Client,
    wanted_secids: &HashSet<SecId>,
) -> Result<(HashMap<SecId, ResolvedSnapshot>, Vec<SecId>), ExampleError> {
    let stock_scope = moex_client
        .stock()
        .expect("stock engine literal must be valid");
    let markets = with_retry(retry_policy(), || stock_scope.markets())?;
    let mut snapshots = HashMap::with_capacity(wanted_secids.len());

    'markets: for market in markets {
        let boards = with_retry(retry_policy(), || {
            moex_client.boards(stock_scope.engine(), market.name())
        })?;

        for board in boards.into_iter().filter(|board| board.is_traded()) {
            let board_snapshots = load_board_snapshots_all_pages(
                moex_client,
                stock_scope.engine().as_str(),
                market.name().as_str(),
                board.boardid().as_str(),
            )?;

            for snapshot in board_snapshots {
                if !wanted_secids.contains(snapshot.secid()) {
                    continue;
                }

                snapshots
                    .entry(snapshot.secid().clone())
                    .or_insert_with(|| ResolvedSnapshot {
                        market: market.name().as_str().to_owned().into_boxed_str(),
                        board: board.boardid().as_str().to_owned().into_boxed_str(),
                        lot_size: snapshot.lot_size(),
                        last: snapshot.last(),
                    });
            }

            if snapshots.len() == wanted_secids.len() {
                break 'markets;
            }
        }
    }

    let missing_mapping = wanted_secids
        .iter()
        .filter(|secid| !snapshots.contains_key(*secid))
        .cloned()
        .collect();

    Ok((snapshots, missing_mapping))
}

fn load_board_snapshots_all_pages(
    moex_client: &Client,
    engine: &str,
    market: &str,
    board: &str,
) -> Result<Vec<SecuritySnapshot>, MoexError> {
    let endpoint = format!("engines/{engine}/markets/{market}/boards/{board}/securities.json");
    let mut start = 0_u32;
    let mut snapshots = Vec::new();
    let mut first_secid_on_previous_page = None;
    let limit = BOARD_SNAPSHOTS_PAGE_LIMIT.to_string();

    loop {
        let payload = with_retry(retry_policy(), || {
            moex_client
                .raw()
                .path(endpoint.as_str())
                .only("securities,marketdata")
                .columns("securities", "SECID,LOTSIZE")
                .columns("marketdata", "SECID,LAST")
                .param("start", start.to_string())
                .param("limit", limit.as_str())
                .send_payload()
        })?;

        let (page, first_secid_on_page) = parse_board_snapshots_page(&payload, endpoint.as_str())?;
        if page.is_empty() {
            break;
        }

        if let (Some(previous), Some(current)) =
            (&first_secid_on_previous_page, &first_secid_on_page)
            && previous == current
        {
            break;
        }
        first_secid_on_previous_page = first_secid_on_page;

        let page_len = u32::try_from(page.len()).map_err(|_| MoexError::PaginationOverflow {
            endpoint: endpoint.clone().into_boxed_str(),
            start,
            limit: BOARD_SNAPSHOTS_PAGE_LIMIT,
        })?;

        snapshots.extend(page);
        if page_len < BOARD_SNAPSHOTS_PAGE_LIMIT {
            break;
        }
        start = start
            .checked_add(page_len)
            .ok_or_else(|| MoexError::PaginationOverflow {
                endpoint: endpoint.clone().into_boxed_str(),
                start,
                limit: BOARD_SNAPSHOTS_PAGE_LIMIT,
            })?;
    }

    Ok(snapshots)
}

fn parse_board_snapshots_page(
    payload: &str,
    endpoint: &str,
) -> Result<(Vec<SecuritySnapshot>, Option<SecId>), MoexError> {
    let mut tables = decode::raw_tables_json(payload, endpoint)?;
    let security_rows: Vec<RawSecuritySnapshotRow> = tables.take_rows("securities")?;
    let marketdata_rows: Vec<RawMarketdataSnapshotRow> = tables.take_rows("marketdata")?;

    let mut marketdata_by_secid = HashMap::with_capacity(marketdata_rows.len());
    for (row, marketdata) in marketdata_rows.into_iter().enumerate() {
        marketdata_by_secid.insert(marketdata.secid, (row, marketdata.last));
    }

    let mut first_secid_on_page = None;
    let mut snapshots = Vec::with_capacity(security_rows.len().max(marketdata_by_secid.len()));

    for (row, security) in security_rows.into_iter().enumerate() {
        let last = marketdata_by_secid
            .remove(security.secid.as_str())
            .and_then(|(_, last)| last);
        let snapshot = SecuritySnapshot::try_new(security.secid, security.lot_size, last).map_err(
            |source| MoexError::InvalidSecuritySnapshot {
                endpoint: endpoint.to_owned().into_boxed_str(),
                table: "securities",
                row,
                source,
            },
        )?;
        if first_secid_on_page.is_none() {
            first_secid_on_page = Some(snapshot.secid().clone());
        }
        snapshots.push(snapshot);
    }

    for (secid, (row, last)) in marketdata_by_secid {
        let snapshot = SecuritySnapshot::try_new(secid, None, last).map_err(|source| {
            MoexError::InvalidSecuritySnapshot {
                endpoint: endpoint.to_owned().into_boxed_str(),
                table: "marketdata",
                row,
                source,
            }
        })?;
        if first_secid_on_page.is_none() {
            first_secid_on_page = Some(snapshot.secid().clone());
        }
        snapshots.push(snapshot);
    }

    Ok((snapshots, first_secid_on_page))
}

#[derive(Debug, serde::Deserialize)]
struct RawSecuritySnapshotRow {
    #[serde(rename = "SECID")]
    secid: String,
    #[serde(rename = "LOTSIZE", default)]
    lot_size: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
struct RawMarketdataSnapshotRow {
    #[serde(rename = "SECID")]
    secid: String,
    #[serde(rename = "LAST", default)]
    last: Option<f64>,
}

fn retry_policy() -> RetryPolicy {
    RetryPolicy::new(
        NonZeroU32::new(REQUEST_RETRIES)
            .expect("REQUEST_RETRIES constant must be greater than zero"),
    )
    .with_delay(Duration::from_millis(RETRY_DELAY_MILLIS))
}

fn print_dump(
    index_dumps: &[IndexDump],
    snapshots: &HashMap<SecId, ResolvedSnapshot>,
    missing_mapping: &[SecId],
) {
    println!("active indexes: {}", index_dumps.len());

    let total_components = index_dumps
        .iter()
        .map(|index| index.components.len())
        .sum::<usize>();
    println!("total components across active indexes: {total_components}");
    println!("resolved security snapshots: {}", snapshots.len());

    if !missing_mapping.is_empty() {
        let preview = missing_mapping
            .iter()
            .take(10)
            .map(SecId::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "warning: {} securities have no stock snapshot data (first 10): {preview}",
            missing_mapping.len()
        );
    }

    for index in index_dumps {
        println!();
        println!("=== {} | {} ===", index.index_id, index.short_name);
        if let Some(first_component) = index.components.first() {
            println!(
                "trade_session_date={} tradingsession={} components={}",
                first_component.trade_session_date(),
                first_component.tradingsession(),
                index.components.len()
            );
        } else {
            println!("components=0");
        }

        println!("SECID\tTICKER\tSHORTNAME\tWEIGHT\tLAST\tLOTSIZE\tMARKET\tBOARD");
        for component in &index.components {
            let snapshot = snapshots.get(component.secid());

            let last = format_optional_f64(snapshot.and_then(|item| item.last));
            let lot_size = format_optional_u32(snapshot.and_then(|item| item.lot_size));
            let market = snapshot.map_or("N/A", |item| item.market.as_ref());
            let board = snapshot.map_or("N/A", |item| item.board.as_ref());

            println!(
                "{}\t{}\t{}\t{:.6}\t{}\t{}\t{}\t{}",
                component.secid(),
                component.ticker(),
                component.shortnames(),
                component.weight(),
                last,
                lot_size,
                market,
                board,
            );
        }
    }
}

fn format_optional_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "N/A".to_owned())
}

fn format_optional_u32(value: Option<u32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "N/A".to_owned())
}

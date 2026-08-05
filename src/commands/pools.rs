use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolToken {
    /// Token contract address (DexPaprika uses "id" for address in pool tokens)
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolDetailPeriod {
    pub last_price_usd_change: Option<f64>,
    pub volume_usd: Option<f64>,
    pub buys: Option<i64>,
    pub sells: Option<i64>,
    pub txns: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolDetailPriceStats {
    pub high: Option<f64>,
    pub low: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolDetail {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    pub factory_id: Option<String>,
    #[serde(default)]
    pub fee: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub created_at_block_number: Option<i64>,
    pub last_price: Option<f64>,
    pub last_price_usd: Option<f64>,
    pub price_time: Option<String>,
    pub price_stats: Option<PoolDetailPriceStats>,
    pub token_reserves: Option<serde_json::Value>,
    pub tokens: Option<Vec<PoolToken>>,
    #[serde(rename = "24h")]
    pub h24: Option<PoolDetailPeriod>,
    #[serde(rename = "6h")]
    pub h6: Option<PoolDetailPeriod>,
    #[serde(rename = "1h")]
    pub h1: Option<PoolDetailPeriod>,
    #[serde(rename = "30m")]
    pub m30: Option<PoolDetailPeriod>,
    #[serde(rename = "15m")]
    pub m15: Option<PoolDetailPeriod>,
    #[serde(rename = "5m")]
    pub m5: Option<PoolDetailPeriod>,
}

/// Wrapper for paginated transaction responses
#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<PoolTransaction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolTransaction {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub token_0: Option<String>,
    pub token_0_symbol: Option<String>,
    pub token_1: Option<String>,
    pub token_1_symbol: Option<String>,
    pub amount_0: Option<serde_json::Value>,
    pub amount_1: Option<serde_json::Value>,
    pub volume_0: Option<f64>,
    pub volume_1: Option<f64>,
    pub price_0_usd: Option<f64>,
    pub price_1_usd: Option<f64>,
    pub created_at: Option<String>,
    #[serde(flatten)]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolOhlcv {
    pub time_open: Option<String>,
    pub time_close: Option<String>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: Option<f64>,
}

/// Unified response for the cursor-paginated `/networks/{network}/pools/search`
/// endpoint. Backs both the pool list and the pool filter commands.
#[derive(Debug, Deserialize, Serialize)]
pub struct PoolSearchResponse {
    #[serde(default)]
    pub results: Vec<PoolSearchItem>,
    pub has_next_page: Option<bool>,
    pub next_cursor: Option<String>,
}

/// A single result item from `/networks/{network}/pools/search`.
#[derive(Debug, Deserialize, Serialize)]
pub struct PoolSearchItem {
    /// Pool address (the search endpoint returns it under "id", not "address").
    pub id: Option<String>,
    pub chain: Option<String>,
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    #[serde(default)]
    pub fee: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub created_at_block_number: Option<i64>,
    pub volume_usd_24h: Option<f64>,
    pub volume_usd_7d: Option<f64>,
    pub volume_usd_30d: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub transactions_24h: Option<i64>,
    pub price_usd: Option<f64>,
    pub price_change_percentage_5m: Option<f64>,
    pub price_change_percentage_1h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub tokens: Option<Vec<PoolToken>>,
}

pub async fn execute_pool_filter(
    client: &ApiClient,
    network: &str,
    volume_24h_min: Option<f64>,
    volume_24h_max: Option<f64>,
    volume_7d_min: Option<f64>,
    volume_7d_max: Option<f64>,
    liquidity_usd_min: Option<f64>,
    liquidity_usd_max: Option<f64>,
    txns_24h_min: Option<u64>,
    created_after: Option<u64>,
    created_before: Option<u64>,
    sort_by: &str,
    sort_dir: &str,
    limit: usize,
    _page: usize,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let order_by = crate::commands::search_mapping::map_pool_sort_field(sort_by);
    // Search is cursor-paginated: no "page" param. "order_by" is the sort field,
    // "sort" the direction. Legacy filter param names are mapped to canonical.
    let mut params: Vec<(&str, String)> = vec![
        ("limit", limit_str),
        ("order_by", order_by.to_string()),
        ("sort", sort_dir.to_string()),
    ];
    if let Some(v) = volume_24h_min {
        params.push(("volume_usd_24h_min", v.to_string()));
    }
    if let Some(v) = volume_24h_max {
        params.push(("volume_usd_24h_max", v.to_string()));
    }
    if let Some(v) = volume_7d_min {
        params.push(("volume_usd_7d_min", v.to_string()));
    }
    if let Some(v) = volume_7d_max {
        params.push(("volume_usd_7d_max", v.to_string()));
    }
    if let Some(v) = liquidity_usd_min {
        params.push(("liquidity_usd_min", v.to_string()));
    }
    if let Some(v) = liquidity_usd_max {
        params.push(("liquidity_usd_max", v.to_string()));
    }
    if let Some(v) = txns_24h_min {
        params.push(("txns_24h_min", v.to_string()));
    }
    if let Some(v) = created_after {
        params.push(("created_after", v.to_string()));
    }
    if let Some(v) = created_before {
        params.push(("created_before", v.to_string()));
    }

    let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let resp: PoolSearchResponse = client
        .dexpaprika_get(&format!("/networks/{network}/pools/search"), &param_refs)
        .await?;

    match output {
        OutputFormat::Table => {
            crate::output::pools::print_pool_filter_table(&resp.results);
            crate::output::print_more_results_hint(resp.has_next_page, resp.next_cursor.as_deref());
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/networks/{network}/pools/search"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

pub async fn execute_pools(
    client: &ApiClient,
    network: &str,
    limit: usize,
    _page: usize,
    order_by: &str,
    sort: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let order_by = crate::commands::search_mapping::map_pool_sort_field(order_by);
    // Search is cursor-paginated: drop "page", map the sort field to canonical.
    let resp: PoolSearchResponse = client
        .dexpaprika_get(
            &format!("/networks/{network}/pools/search"),
            &[
                ("limit", limit_str.as_str()),
                ("order_by", order_by),
                ("sort", sort),
            ],
        )
        .await?;
    match output {
        OutputFormat::Table => {
            crate::output::pools::print_pool_search_table(&resp.results);
            crate::output::print_more_results_hint(resp.has_next_page, resp.next_cursor.as_deref());
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/networks/{network}/pools/search"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

pub async fn execute_pool_detail(
    client: &ApiClient,
    network: &str,
    pool_address: &str,
    inversed: bool,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if inversed {
        params.push(("inversed", "true"));
    }
    let pool: PoolDetail = client
        .dexpaprika_get(
            &format!("/networks/{network}/pools/{pool_address}"),
            &params,
        )
        .await?;
    match output {
        OutputFormat::Table => crate::output::pools::print_pool_detail(&pool),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &pool,
                crate::output::ResponseMeta::dexpaprika(&format!("/pool/{network}/{pool_address}")),
                raw,
            )?;
        }
    }
    Ok(())
}

/// List the pools of a single DEX.
///
/// `/networks/{network}/dexes/{dex}/pools` was removed and returns HTTP 410, so
/// the DEX moved out of the path and into the `dex_name` query parameter on
/// `/networks/{network}/pools/search`. The parameter resolves both forms, the
/// dex id ("uniswap_v3") and the display name ("Uniswap V3"); the CLI passes
/// whatever the user typed straight through. Search is cursor-paginated, so the
/// old `page` number is gone and `cursor` takes its place.
pub async fn execute_dex_pools(
    client: &ApiClient,
    network: &str,
    dex: &str,
    limit: usize,
    cursor: Option<&str>,
    order_by: &str,
    sort: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let order_by = crate::commands::search_mapping::map_pool_sort_field(order_by);
    let mut params: Vec<(&str, &str)> = vec![
        ("limit", limit_str.as_str()),
        ("dex_name", dex),
        ("order_by", order_by),
        ("sort", sort),
    ];
    if let Some(c) = cursor {
        params.push(("cursor", c));
    }
    let resp: PoolSearchResponse = client
        .dexpaprika_get(&format!("/networks/{network}/pools/search"), &params)
        .await?;
    match output {
        OutputFormat::Table => {
            crate::output::pools::print_pool_search_table(&resp.results);
            crate::output::print_more_results_hint(resp.has_next_page, resp.next_cursor.as_deref());
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/networks/{network}/pools/search"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

pub async fn execute_transactions(
    client: &ApiClient,
    network: &str,
    pool_address: &str,
    limit: usize,
    cursor: Option<&str>,
    from: Option<i64>,
    to: Option<i64>,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let from_str = from.map(|f| f.to_string());
    let to_str = to.map(|t| t.to_string());
    let mut params: Vec<(&str, &str)> = vec![("limit", &limit_str)];
    if let Some(c) = cursor {
        params.push(("cursor", c));
    }
    if let Some(ref f) = from_str {
        params.push(("from", f));
    }
    if let Some(ref t) = to_str {
        params.push(("to", t));
    }
    let resp: TransactionsResponse = client
        .dexpaprika_get(
            &format!("/networks/{network}/pools/{pool_address}/transactions"),
            &params,
        )
        .await?;
    let txs = resp.transactions;
    match output {
        OutputFormat::Table => crate::output::pools::print_transactions_table(&txs),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &txs,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/pool/{network}/{pool_address}/transactions"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

pub async fn execute_ohlcv(
    client: &ApiClient,
    network: &str,
    pool_address: &str,
    start: &str,
    end: Option<&str>,
    interval: &str,
    limit: usize,
    inversed: bool,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    // Validate start date format
    let is_unix = start.chars().all(|c| c.is_ascii_digit());
    let is_date =
        start.len() == 10 && start.chars().nth(4) == Some('-') && start.chars().nth(7) == Some('-');
    let is_rfc3339 = start.contains('T');
    if !is_unix && !is_date && !is_rfc3339 {
        anyhow::bail!(
            "Invalid --start format: \"{start}\". Use yyyy-mm-dd, unix timestamp, or RFC3339."
        );
    }

    let limit_str = limit.to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("start", start),
        ("interval", interval),
        ("limit", &limit_str),
    ];
    if let Some(e) = end {
        params.push(("end", e));
    }
    if inversed {
        params.push(("inversed", "true"));
    }

    let data: Vec<PoolOhlcv> = client
        .dexpaprika_get(
            &format!("/networks/{network}/pools/{pool_address}/ohlcv"),
            &params,
        )
        .await?;
    match output {
        OutputFormat::Table => crate::output::pools::print_pool_ohlcv_table(&data),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &data,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/pool/{network}/{pool_address}/ohlcv"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One result object, copied verbatim from a live
    /// `/networks/ethereum/pools/search?limit=2&dex_name=curve` response
    /// captured on 2026-08-05. Field names come off the wire, not from docs.
    const LIVE_DEX_POOLS_SAMPLE: &str = r#"{
      "results": [
        {
          "id": "0x4f493b7de8aac7d55f71853688b1f7c8f0243c85",
          "dex_id": "curve",
          "dex_name": "Curve",
          "chain": "ethereum",
          "volume_usd_24h": 15883391.558251368,
          "created_at": "2025-01-25T17:20:47Z",
          "created_at_block_number": 21702976,
          "transactions_24h": 289,
          "price_usd": 0.9995787501356217,
          "price_change_percentage_5m": null,
          "price_change_percentage_1h": 0.02422482089565938,
          "price_change_percentage_6h": 0.009802157529374174,
          "price_change_percentage_24h": 0.007018797950998323,
          "fee": null,
          "volume_usd_7d": 31781851.73428885,
          "volume_usd_30d": 136889876.39037386,
          "liquidity_usd": 7407910.088430515,
          "tokens": [
            {"id": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", "chain": "ethereum", "has_image": true}
          ]
        }
      ],
      "has_next_page": true,
      "next_cursor": "eyJjaGFpbiI6ImV0aGVyZXVtIn0",
      "query": {"network": "ethereum", "limit": 2, "dex_name": "curve", "order_by": "volume_usd_24h"}
    }"#;

    #[test]
    fn dex_pools_payload_decodes_from_the_search_envelope() {
        let resp: PoolSearchResponse =
            serde_json::from_str(LIVE_DEX_POOLS_SAMPLE).expect("live sample must decode");

        // The envelope is results/has_next_page/next_cursor, not pools/page_info.
        assert_eq!(resp.results.len(), 1);
        assert_eq!(resp.has_next_page, Some(true));
        assert_eq!(
            resp.next_cursor.as_deref(),
            Some("eyJjaGFpbiI6ImV0aGVyZXVtIn0")
        );

        // The 24h volume field is volume_usd_24h. A bare volume_usd would decode
        // as None here, which is exactly the silent breakage this test guards.
        let pool = &resp.results[0];
        assert_eq!(pool.volume_usd_24h, Some(15883391.558251368));
        assert_eq!(pool.dex_id.as_deref(), Some("curve"));
        assert_eq!(
            pool.id.as_deref(),
            Some("0x4f493b7de8aac7d55f71853688b1f7c8f0243c85")
        );
        assert_eq!(pool.transactions_24h, Some(289));
    }

    #[test]
    fn dex_pools_default_sort_maps_to_the_canonical_search_field() {
        // The dex-pools --order-by default is still the legacy "volume_usd";
        // pools/search rejects it, so it has to be mapped before it goes out.
        assert_eq!(
            crate::commands::search_mapping::map_pool_sort_field("volume_usd"),
            "volume_usd_24h"
        );
    }
}

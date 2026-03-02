use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

/// Wrapper for paginated pool list responses
#[derive(Debug, Deserialize, Serialize)]
pub struct PoolsResponse {
    pub pools: Vec<PoolListItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolListItem {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    #[serde(default)]
    pub fee: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub created_at_block_number: Option<i64>,
    pub volume_usd: Option<f64>,
    pub transactions: Option<serde_json::Value>,
    pub price_usd: Option<f64>,
    pub last_price_change_usd_5m: Option<f64>,
    pub last_price_change_usd_1h: Option<f64>,
    pub last_price_change_usd_24h: Option<f64>,
    pub tokens: Option<Vec<PoolToken>>,
}

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

/// Wrapper for pool filter responses (uses "results" key, not "pools")
#[derive(Debug, Deserialize, Serialize)]
pub struct PoolFilterResponse {
    pub results: Vec<PoolFilterItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolFilterItem {
    pub address: Option<String>,
    pub chain: Option<String>,
    pub dex_id: Option<String>,
    pub volume_usd_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub txns_24h: Option<i64>,
    pub created_at: Option<String>,
}

pub async fn execute_pool_filter(
    client: &ApiClient,
    network: &str,
    volume_24h_min: Option<f64>,
    volume_24h_max: Option<f64>,
    txns_24h_min: Option<u64>,
    created_after: Option<u64>,
    created_before: Option<u64>,
    sort_by: &str,
    sort_dir: &str,
    limit: usize,
    page: usize,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let page_str = page.to_string();
    let mut params: Vec<(&str, String)> = vec![
        ("limit", limit_str),
        ("page", page_str),
        ("sort_by", sort_by.to_string()),
        ("sort_dir", sort_dir.to_string()),
    ];
    if let Some(v) = volume_24h_min {
        params.push(("volume_24h_min", v.to_string()));
    }
    if let Some(v) = volume_24h_max {
        params.push(("volume_24h_max", v.to_string()));
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
    let resp: PoolFilterResponse = client.dexpaprika_get(
        &format!("/networks/{network}/pools/filter"),
        &param_refs,
    ).await?;
    let results = resp.results;
    match output {
        OutputFormat::Table => crate::output::pools::print_pool_filter_table(&results),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&results, crate::output::ResponseMeta::dexpaprika(&format!("/network/{network}/pools/filter")), raw)?;
        }
    }
    Ok(())
}

pub async fn execute_pools(
    client: &ApiClient,
    network: &str,
    limit: usize,
    page: usize,
    order_by: &str,
    sort: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let page_str = page.to_string();
    let resp: PoolsResponse = client.dexpaprika_get(
        &format!("/networks/{network}/pools"),
        &[("limit", &limit_str), ("page", &page_str), ("order_by", order_by), ("sort", sort)],
    ).await?;
    let pools = resp.pools;
    match output {
        OutputFormat::Table => crate::output::pools::print_pools_table(&pools),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&pools, crate::output::ResponseMeta::dexpaprika(&format!("/network/{network}/pools")), raw)?;
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
    let pool: PoolDetail = client.dexpaprika_get(
        &format!("/networks/{network}/pools/{pool_address}"),
        &params,
    ).await?;
    match output {
        OutputFormat::Table => crate::output::pools::print_pool_detail(&pool),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&pool, crate::output::ResponseMeta::dexpaprika(&format!("/pool/{network}/{pool_address}")), raw)?;
        }
    }
    Ok(())
}

pub async fn execute_dex_pools(
    client: &ApiClient,
    network: &str,
    dex: &str,
    limit: usize,
    page: usize,
    order_by: &str,
    sort: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let page_str = page.to_string();
    let resp: PoolsResponse = client.dexpaprika_get(
        &format!("/networks/{network}/dexes/{dex}/pools"),
        &[("limit", &limit_str), ("page", &page_str), ("order_by", order_by), ("sort", sort)],
    ).await?;
    let pools = resp.pools;
    match output {
        OutputFormat::Table => crate::output::pools::print_pools_table(&pools),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&pools, crate::output::ResponseMeta::dexpaprika(&format!("/dex/{network}/{dex}/pools")), raw)?;
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
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", &limit_str)];
    if let Some(c) = cursor {
        params.push(("cursor", c));
    }
    let resp: TransactionsResponse = client.dexpaprika_get(
        &format!("/networks/{network}/pools/{pool_address}/transactions"),
        &params,
    ).await?;
    let txs = resp.transactions;
    match output {
        OutputFormat::Table => crate::output::pools::print_transactions_table(&txs),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&txs, crate::output::ResponseMeta::dexpaprika(&format!("/pool/{network}/{pool_address}/transactions")), raw)?;
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
    let is_date = start.len() == 10 && start.chars().nth(4) == Some('-') && start.chars().nth(7) == Some('-');
    let is_rfc3339 = start.contains('T');
    if !is_unix && !is_date && !is_rfc3339 {
        anyhow::bail!("Invalid --start format: \"{start}\". Use yyyy-mm-dd, unix timestamp, or RFC3339.");
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

    let data: Vec<PoolOhlcv> = client.dexpaprika_get(
        &format!("/networks/{network}/pools/{pool_address}/ohlcv"),
        &params,
    ).await?;
    match output {
        OutputFormat::Table => crate::output::pools::print_pool_ohlcv_table(&data),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&data, crate::output::ResponseMeta::dexpaprika(&format!("/pool/{network}/{pool_address}/ohlcv")), raw)?;
        }
    }
    Ok(())
}

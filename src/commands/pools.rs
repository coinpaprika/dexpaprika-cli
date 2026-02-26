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
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    pub chain: Option<String>,
    pub tokens: Option<Vec<PoolToken>>,
    pub price_usd: Option<f64>,
    pub volume_usd: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub transactions: Option<serde_json::Value>,
    pub last_price_change_usd_24h: Option<f64>,
    pub created_at: Option<String>,
    #[serde(default)]
    pub fee: Option<serde_json::Value>,
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
pub struct PoolDetail {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    pub tokens: Option<Vec<PoolToken>>,
    pub last_price_usd: Option<f64>,
    pub created_at: Option<String>,
    #[serde(rename = "24h")]
    pub h24: Option<PoolDetailPeriod>,
    #[serde(flatten)]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
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
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let page_str = page.to_string();
    let resp: PoolsResponse = client.dexpaprika_get(
        &format!("/networks/{network}/dexes/{dex}/pools"),
        &[("limit", &limit_str), ("page", &page_str)],
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

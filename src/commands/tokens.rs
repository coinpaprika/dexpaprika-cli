use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenDetail {
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub chain: Option<String>,
    pub decimals: Option<i64>,
    pub total_supply: Option<f64>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub has_image: Option<bool>,
    pub added_at: Option<String>,
    pub price_stats: Option<TokenPriceStats>,
    pub summary: Option<TokenSummary>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPriceStats {
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub ath: Option<f64>,
    pub ath_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenSummary {
    pub chain: Option<String>,
    pub id: Option<String>,
    pub price_usd: Option<f64>,
    pub fdv: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub pools: Option<i64>,
    #[serde(rename = "24h")]
    pub h24: Option<TokenPeriodStats>,
    #[serde(rename = "6h")]
    pub h6: Option<TokenPeriodStats>,
    #[serde(rename = "1h")]
    pub h1: Option<TokenPeriodStats>,
    #[serde(rename = "30m")]
    pub m30: Option<TokenPeriodStats>,
    #[serde(rename = "15m")]
    pub m15: Option<TokenPeriodStats>,
    #[serde(rename = "5m")]
    pub m5: Option<TokenPeriodStats>,
    #[serde(rename = "1m")]
    pub m1: Option<TokenPeriodStats>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPeriodStats {
    pub volume: Option<f64>,
    pub volume_usd: Option<f64>,
    pub sells: Option<i64>,
    pub buys: Option<i64>,
    pub txns: Option<i64>,
    pub buy_usd: Option<f64>,
    pub sell_usd: Option<f64>,
    pub last_price_usd_change: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPoolItem {
    pub id: Option<String>,
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    pub tokens: Option<Vec<TokenPoolToken>>,
    pub price_usd: Option<f64>,
    pub volume_usd: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub last_price_change_usd_24h: Option<f64>,
    pub created_at: Option<String>,
}

/// Wrapper for paginated token-pools responses
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPoolsResponse {
    pub pools: Vec<TokenPoolItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPoolToken {
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPrice {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub price_usd: Option<f64>,
}

// --- Token filter types (for GET /networks/{network}/tokens/filter) ---

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenFilterResponse {
    pub results: Vec<TokenFilterResult>,
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenFilterResult {
    pub chain: Option<String>,
    pub address: Option<String>,
    pub price_usd: Option<f64>,
    pub volume_usd_24h: Option<f64>,
    pub volume_usd_7d: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub fdv_usd: Option<f64>,
    pub txns_24h: Option<i64>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PageInfo {
    pub limit: Option<i64>,
    pub page: Option<i64>,
    pub total_items: Option<i64>,
    pub total_pages: Option<i64>,
}

// --- Top tokens API types (for GET /networks/{network}/tokens/top) ---

#[derive(Debug, Deserialize, Serialize)]
pub struct TopTokensResponse {
    pub tokens: Vec<TopTokenApiItem>,
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TopTokenApiItem {
    pub address: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub chain: Option<String>,
    pub decimals: Option<i64>,
    pub has_image: Option<bool>,
    pub price_usd: Option<f64>,
    pub fdv: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub pools: Option<i64>,
    #[serde(rename = "24h")]
    pub h24: Option<TopTokenTimeData>,
    #[serde(rename = "1h")]
    pub h1: Option<TopTokenTimeData>,
    #[serde(rename = "5m")]
    pub m5: Option<TopTokenTimeData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TopTokenTimeData {
    pub volume_usd: Option<f64>,
    pub buys: Option<i64>,
    pub sells: Option<i64>,
    pub txns: Option<i64>,
    pub last_price_usd_change: Option<f64>,
}

/// Summary of a token for the top-tokens ranking (derived from full TokenDetail)
#[derive(Debug, Serialize)]
pub struct TopTokenEntry {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub price_usd: Option<f64>,
    pub volume_usd_24h: Option<f64>,
    pub change_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub buys_24h: Option<i64>,
    pub sells_24h: Option<i64>,
    pub txns_24h: Option<i64>,
    pub fdv: Option<f64>,
    pub pools: Option<i64>,
}

pub async fn execute_top_tokens(
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
    let resp: TopTokensResponse = client.dexpaprika_get(
        &format!("/networks/{network}/tokens/top"),
        &[("limit", &limit_str), ("page", &page_str), ("order_by", order_by), ("sort", sort)],
    ).await?;

    let entries: Vec<TopTokenEntry> = resp.tokens.iter().map(|t| {
        let (vol, change, buys, sells, txns) = match &t.h24 {
            Some(h) => (h.volume_usd, h.last_price_usd_change, h.buys, h.sells, h.txns),
            None => (None, None, None, None, None),
        };
        TopTokenEntry {
            address: t.address.clone().unwrap_or_default(),
            name: t.name.clone().unwrap_or_default(),
            symbol: t.symbol.clone().unwrap_or_default(),
            price_usd: t.price_usd,
            volume_usd_24h: vol,
            change_24h: change,
            liquidity_usd: t.liquidity_usd,
            buys_24h: buys,
            sells_24h: sells,
            txns_24h: txns,
            fdv: t.fdv,
            pools: t.pools,
        }
    }).collect();

    match output {
        OutputFormat::Table => {
            crate::output::tokens::print_top_tokens_table(&entries);
            if let Some(pi) = &resp.page_info {
                println!("  Page {}/{} ({} tokens total)",
                    pi.page.unwrap_or(0), pi.total_pages.unwrap_or(0), pi.total_items.unwrap_or(0));
            }
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&resp, crate::output::ResponseMeta::dexpaprika(&format!("/networks/{network}/tokens/top")), raw)?;
        }
    }
    Ok(())
}

pub async fn execute_token(client: &ApiClient, network: &str, token_address: &str, output: OutputFormat, raw: bool) -> Result<()> {
    let token: TokenDetail = client.dexpaprika_get(
        &format!("/networks/{network}/tokens/{token_address}"),
        &[],
    ).await?;
    match output {
        OutputFormat::Table => crate::output::tokens::print_token_detail(&token),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&token, crate::output::ResponseMeta::dexpaprika(&format!("/token/{network}/{token_address}")), raw)?;
        }
    }
    Ok(())
}

pub async fn execute_token_pools(
    client: &ApiClient,
    network: &str,
    token_address: &str,
    limit: usize,
    page: usize,
    order_by: &str,
    sort: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let page_str = page.to_string();
    let resp: TokenPoolsResponse = client.dexpaprika_get(
        &format!("/networks/{network}/tokens/{token_address}/pools"),
        &[("limit", &limit_str), ("page", &page_str), ("order_by", order_by), ("sort", sort)],
    ).await?;
    let pools = resp.pools;
    match output {
        OutputFormat::Table => crate::output::tokens::print_token_pools_table(&pools),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&pools, crate::output::ResponseMeta::dexpaprika(&format!("/token/{network}/{token_address}/pools")), raw)?;
        }
    }
    Ok(())
}

pub async fn execute_prices(client: &ApiClient, network: &str, tokens: &str, output: OutputFormat, raw: bool) -> Result<()> {
    let prices: Vec<TokenPrice> = client.dexpaprika_get(
        &format!("/networks/{network}/multi/prices"),
        &[("tokens", tokens)],
    ).await?;

    if prices.is_empty() {
        anyhow::bail!("No price data found. Check that the token addresses are valid on {network}.");
    }

    match output {
        OutputFormat::Table => crate::output::tokens::print_prices_table(&prices),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&prices, crate::output::ResponseMeta::dexpaprika(&format!("/network/{network}/prices")), raw)?;
        }
    }
    Ok(())
}

pub async fn execute_filter_tokens(
    client: &ApiClient,
    network: &str,
    limit: usize,
    page: usize,
    sort_by: &str,
    sort_dir: &str,
    volume_24h_min: Option<f64>,
    volume_24h_max: Option<f64>,
    liquidity_usd_min: Option<f64>,
    fdv_min: Option<f64>,
    fdv_max: Option<f64>,
    txns_24h_min: Option<u64>,
    created_after: Option<u64>,
    created_before: Option<u64>,
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
    if let Some(v) = volume_24h_min { params.push(("volume_24h_min", v.to_string())); }
    if let Some(v) = volume_24h_max { params.push(("volume_24h_max", v.to_string())); }
    if let Some(v) = liquidity_usd_min { params.push(("liquidity_usd_min", v.to_string())); }
    if let Some(v) = fdv_min { params.push(("fdv_min", v.to_string())); }
    if let Some(v) = fdv_max { params.push(("fdv_max", v.to_string())); }
    if let Some(v) = txns_24h_min { params.push(("txns_24h_min", v.to_string())); }
    if let Some(v) = created_after { params.push(("created_after", v.to_string())); }
    if let Some(v) = created_before { params.push(("created_before", v.to_string())); }

    let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let resp: TokenFilterResponse = client.dexpaprika_get(
        &format!("/networks/{network}/tokens/filter"),
        &param_refs,
    ).await?;

    match output {
        OutputFormat::Table => {
            crate::output::tokens::print_token_filter_table(&resp.results);
            if let Some(pi) = &resp.page_info {
                println!("  Page {}/{} ({} tokens total)",
                    pi.page.unwrap_or(0), pi.total_pages.unwrap_or(0), pi.total_items.unwrap_or(0));
            }
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&resp, crate::output::ResponseMeta::dexpaprika(&format!("/networks/{network}/tokens/filter")), raw)?;
        }
    }
    Ok(())
}

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
    pub summary: Option<TokenSummary>,
    #[serde(flatten)]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenSummary {
    pub price_usd: Option<f64>,
    pub liquidity_usd: Option<f64>,
    #[serde(rename = "24h")]
    pub h24: Option<TokenPeriodStats>,
    #[serde(rename = "6h")]
    pub h6: Option<TokenPeriodStats>,
    #[serde(rename = "1h")]
    pub h1: Option<TokenPeriodStats>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPeriodStats {
    pub volume_usd: Option<f64>,
    pub last_price_usd_change: Option<f64>,
    pub buys: Option<i64>,
    pub sells: Option<i64>,
    pub txns: Option<i64>,
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

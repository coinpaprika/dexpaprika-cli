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
    pub telegram: Option<String>,
    pub twitter: Option<String>,
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
pub struct TokenPrice {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub price_usd: Option<f64>,
}

// --- Unified token search types (for GET /networks/{network}/tokens/search) ---

/// Unified response for the cursor-paginated `/networks/{network}/tokens/search`
/// endpoint. Backs both the top-tokens and the token-filter commands.
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenSearchResponse {
    #[serde(default)]
    pub results: Vec<TokenSearchItem>,
    pub has_next_page: Option<bool>,
    pub next_cursor: Option<String>,
}

/// A single result item from `/networks/{network}/tokens/search`. The shape is
/// flat: there is no name, symbol, buys/sells, pools count, or nested time
/// buckets, unlike the removed list endpoints.
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenSearchItem {
    pub address: Option<String>,
    pub chain: Option<String>,
    pub created_at: Option<String>,
    pub price_usd: Option<f64>,
    pub volume_usd_24h: Option<f64>,
    pub volume_usd_7d: Option<f64>,
    pub volume_usd_30d: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub fdv_usd: Option<f64>,
    pub txns_24h: Option<i64>,
    pub price_change_percentage_24h: Option<f64>,
}

pub async fn execute_top_tokens(
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
    let order_by = crate::commands::search_mapping::map_token_sort_field(order_by);
    // Search is cursor-paginated: drop "page", map the sort field to canonical.
    let resp: TokenSearchResponse = client
        .dexpaprika_get(
            &format!("/networks/{network}/tokens/search"),
            &[
                ("limit", limit_str.as_str()),
                ("order_by", order_by),
                ("sort", sort),
            ],
        )
        .await?;

    match output {
        OutputFormat::Table => {
            crate::output::tokens::print_token_search_table(&resp.results);
            crate::output::print_more_results_hint(resp.has_next_page, resp.next_cursor.as_deref());
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/networks/{network}/tokens/search"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

pub async fn execute_token(
    client: &ApiClient,
    network: &str,
    token_address: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let token: TokenDetail = client
        .dexpaprika_get(&format!("/networks/{network}/tokens/{token_address}"), &[])
        .await?;
    match output {
        OutputFormat::Table => crate::output::tokens::print_token_detail(&token),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &token,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/token/{network}/{token_address}"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

pub async fn execute_token_pools(
    client: &ApiClient,
    network: &str,
    token_address: &str,
    limit: usize,
    _page: usize,
    order_by: &str,
    sort: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let order_by = crate::commands::search_mapping::map_pool_sort_field(order_by);
    // The dedicated /networks/{network}/tokens/{address}/pools endpoint was
    // removed (HTTP 410). Its replacement is /networks/{network}/pools/search
    // with a token_address filter, which restricts results to pools containing
    // that token on that network. The filter is network-scoped only: the
    // cross-network /pools/search accepts token_address but silently ignores
    // it. An unknown address is not an error; it just returns zero results.
    // Search is cursor-paginated, so "page" is dropped, and the sort field is
    // mapped to canonical like the other search-backed commands. The old
    // pair-perspective (reorder) and second-token (address) params have no
    // search equivalent and were never exposed by this command.
    let resp: crate::commands::pools::PoolSearchResponse = client
        .dexpaprika_get(
            &format!("/networks/{network}/pools/search"),
            &[
                ("token_address", token_address),
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

pub async fn execute_prices(
    client: &ApiClient,
    network: &str,
    tokens: &str,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let prices: Vec<TokenPrice> = client
        .dexpaprika_get(
            &format!("/networks/{network}/multi/prices"),
            &[("tokens", tokens)],
        )
        .await?;

    if prices.is_empty() {
        anyhow::bail!(
            "No price data found. Check that the token addresses are valid on {network}."
        );
    }

    match output {
        OutputFormat::Table => crate::output::tokens::print_prices_table(&prices),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &prices,
                crate::output::ResponseMeta::dexpaprika(&format!("/network/{network}/prices")),
                raw,
            )?;
        }
    }
    Ok(())
}

/// Wire names for the token price-change bounds.
///
/// Pulled out of `execute_filter_tokens` so it can be tested. That matters more
/// than usual here: `/networks/{network}/tokens/search` ignores query
/// parameters it does not recognise and still answers `200` with the full
/// unfiltered page, so a misspelled name produces a plausible result set and no
/// error anywhere. A unit test on the literal strings is the only cheap place
/// that catches it.
///
/// 24h is the only window tokens carry. The 6h, 1h and 5m bounds that
/// `pool-filter` takes are deliberately absent: token rows have no such fields,
/// and ordering tokens by any of the three is a `400`.
fn token_price_change_params(
    min: Option<f64>,
    max: Option<f64>,
) -> Vec<(&'static str, String)> {
    let mut params = Vec::new();
    if let Some(v) = min {
        params.push(("price_change_percentage_24h_min", v.to_string()));
    }
    if let Some(v) = max {
        params.push(("price_change_percentage_24h_max", v.to_string()));
    }
    params
}

pub async fn execute_filter_tokens(
    client: &ApiClient,
    network: &str,
    limit: usize,
    _page: usize,
    sort_by: &str,
    sort_dir: &str,
    volume_24h_min: Option<f64>,
    volume_24h_max: Option<f64>,
    liquidity_usd_min: Option<f64>,
    fdv_min: Option<f64>,
    fdv_max: Option<f64>,
    txns_24h_min: Option<u64>,
    price_change_24h_min: Option<f64>,
    price_change_24h_max: Option<f64>,
    created_after: Option<u64>,
    created_before: Option<u64>,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let limit_str = limit.to_string();
    let order_by = crate::commands::search_mapping::map_token_sort_field(sort_by);
    // Search is cursor-paginated: no "page". "order_by"/"sort" replace
    // "sort_by"/"sort_dir", and legacy filter param names map to canonical.
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
    if let Some(v) = liquidity_usd_min {
        params.push(("liquidity_usd_min", v.to_string()));
    }
    if let Some(v) = fdv_min {
        params.push(("fdv_min", v.to_string()));
    }
    if let Some(v) = fdv_max {
        params.push(("fdv_max", v.to_string()));
    }
    params.extend(token_price_change_params(price_change_24h_min, price_change_24h_max));
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

    let resp: TokenSearchResponse = client
        .dexpaprika_get(&format!("/networks/{network}/tokens/search"), &param_refs)
        .await?;

    match output {
        OutputFormat::Table => {
            crate::output::tokens::print_token_search_table(&resp.results);
            crate::output::print_more_results_hint(resp.has_next_page, resp.next_cursor.as_deref());
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/networks/{network}/tokens/search"
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

    /// Verified against api.dexpaprika.com on 2026-08-07. Baseline 24h changes
    /// on ethereum were [95.7, -0.04, -0.02, -0.0, 0.61]; with
    /// --price-change-24h-min 20 the same command returned
    /// [95.7, 36.36, 923.6, 41.72, 2764.61], and with --price-change-24h-max -20
    /// it returned [-99.32, -23.58, -90.79, -54.22, -21.12].
    #[test]
    fn token_price_change_params_carry_the_names_the_api_expects() {
        let params = token_price_change_params(Some(20.0), Some(-20.0));
        let pairs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        assert_eq!(
            pairs,
            vec![
                ("price_change_percentage_24h_min", "20"),
                ("price_change_percentage_24h_max", "-20"),
            ]
        );
    }

    #[test]
    fn unset_token_bounds_send_nothing() {
        assert!(token_price_change_params(None, None).is_empty());
    }
}

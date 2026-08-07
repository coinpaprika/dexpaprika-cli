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
    pub price_change_percentage_6h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub tokens: Option<Vec<PoolToken>>,
}

/// The four price-change windows that `/networks/{network}/pools/search` accepts
/// as bounds, carried together so the CLI flags cannot get transposed on the way
/// through. Values are percentages and negatives are ordinary input: a max of
/// -20 means "down 20% or more".
///
/// Only the three short windows are pools-only. tokens/search takes the 24h
/// bounds as well, and silently ignores the 6h, 1h and 5m ones. Checked on
/// 2026-08-07 against an unfiltered baseline, because an unknown bound comes
/// back as HTTP 200 with a full result set: on ethereum tokens/search,
/// `price_change_percentage_24h_min=20` returned 36.6, 33.2, 1251.0 where the
/// baseline had -0.07, 0.24, -0.01, while `price_change_percentage_6h_min=20`
/// returned the baseline unchanged.
#[derive(Debug, Default, Clone, Copy)]
pub struct PriceChangeBounds {
    pub price_change_24h_min: Option<f64>,
    pub price_change_24h_max: Option<f64>,
    pub price_change_6h_min: Option<f64>,
    pub price_change_6h_max: Option<f64>,
    pub price_change_1h_min: Option<f64>,
    pub price_change_1h_max: Option<f64>,
    pub price_change_5m_min: Option<f64>,
    pub price_change_5m_max: Option<f64>,
}

/// Pair each set bound with the query parameter name the API expects.
///
/// This lives outside `execute_pool_filter` so the eight names can be pinned by
/// a test. Nothing at runtime can catch a typo in them: pools/search answers 200
/// and hands back a full unfiltered set for a parameter it does not recognise,
/// so `price_change_percentage_1hr_min` looks exactly like a working query. The
/// live wire is the authority for what these names are, the tests below are the
/// lock that stops them drifting afterwards.
///
/// f64's Display gives the shortest round-trip form, so a bound of -20.0 goes
/// out as "-20".
pub fn price_change_params(bounds: &PriceChangeBounds) -> Vec<(&'static str, String)> {
    [
        (
            "price_change_percentage_24h_min",
            bounds.price_change_24h_min,
        ),
        (
            "price_change_percentage_24h_max",
            bounds.price_change_24h_max,
        ),
        ("price_change_percentage_6h_min", bounds.price_change_6h_min),
        ("price_change_percentage_6h_max", bounds.price_change_6h_max),
        ("price_change_percentage_1h_min", bounds.price_change_1h_min),
        ("price_change_percentage_1h_max", bounds.price_change_1h_max),
        ("price_change_percentage_5m_min", bounds.price_change_5m_min),
        ("price_change_percentage_5m_max", bounds.price_change_5m_max),
    ]
    .into_iter()
    .filter_map(|(name, value)| value.map(|v| (name, v.to_string())))
    .collect()
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
    price_change: PriceChangeBounds,
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
    params.extend(price_change_params(&price_change));
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
    let resp: PoolsResponse = client
        .dexpaprika_get(
            &format!("/networks/{network}/dexes/{dex}/pools"),
            &[
                ("limit", &limit_str),
                ("page", &page_str),
                ("order_by", order_by),
                ("sort", sort),
            ],
        )
        .await?;
    let pools = resp.pools;
    match output {
        OutputFormat::Table => crate::output::pools::print_pools_table(&pools),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &pools,
                crate::output::ResponseMeta::dexpaprika(&format!("/dex/{network}/{dex}/pools")),
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

    /// Every bound set, each to a different value, so a transposed pair shows up
    /// as a wrong value next to the right name.
    fn all_bounds() -> PriceChangeBounds {
        PriceChangeBounds {
            price_change_24h_min: Some(1.0),
            price_change_24h_max: Some(2.0),
            price_change_6h_min: Some(3.0),
            price_change_6h_max: Some(4.0),
            price_change_1h_min: Some(5.0),
            price_change_1h_max: Some(6.0),
            price_change_5m_min: Some(7.0),
            price_change_5m_max: Some(8.0),
        }
    }

    #[test]
    fn price_change_params_carry_the_names_the_api_expects() {
        let params = price_change_params(&all_bounds());
        let pairs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        assert_eq!(
            pairs,
            vec![
                ("price_change_percentage_24h_min", "1"),
                ("price_change_percentage_24h_max", "2"),
                ("price_change_percentage_6h_min", "3"),
                ("price_change_percentage_6h_max", "4"),
                ("price_change_percentage_1h_min", "5"),
                ("price_change_percentage_1h_max", "6"),
                ("price_change_percentage_5m_min", "7"),
                ("price_change_percentage_5m_max", "8"),
            ]
        );
    }

    /// The same eight names one layer further out, read off the URL reqwest
    /// would actually send. Verified against api.dexpaprika.com on 2026-08-07:
    /// each of these bounds changes the result set, and a misspelling of any of
    /// them returns the unfiltered baseline at HTTP 200.
    #[test]
    fn price_change_bounds_reach_the_query_string() {
        let params = price_change_params(&all_bounds());
        let refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let request = reqwest::Client::new()
            .get("https://api.dexpaprika.com/networks/ethereum/pools/search")
            .query(&refs)
            .build()
            .expect("the request should build");

        assert_eq!(
            request.url().query().expect("a query string"),
            "price_change_percentage_24h_min=1&price_change_percentage_24h_max=2\
             &price_change_percentage_6h_min=3&price_change_percentage_6h_max=4\
             &price_change_percentage_1h_min=5&price_change_percentage_1h_max=6\
             &price_change_percentage_5m_min=7&price_change_percentage_5m_max=8"
        );
    }

    #[test]
    fn negative_bounds_survive_url_encoding() {
        // "down 20% or more over 24h" has to leave as -20, not as %2D20 or 20.
        let bounds = PriceChangeBounds {
            price_change_24h_max: Some(-20.0),
            price_change_5m_min: Some(-1.5),
            ..PriceChangeBounds::default()
        };
        let params = price_change_params(&bounds);
        let refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let request = reqwest::Client::new()
            .get("https://api.dexpaprika.com/networks/ethereum/pools/search")
            .query(&refs)
            .build()
            .expect("the request should build");

        assert_eq!(
            request.url().query().expect("a query string"),
            "price_change_percentage_24h_max=-20&price_change_percentage_5m_min=-1.5"
        );
    }

    #[test]
    fn unset_bounds_send_nothing() {
        assert!(price_change_params(&PriceChangeBounds::default()).is_empty());
    }
}

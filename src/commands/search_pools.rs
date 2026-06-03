//! `search-pools` subcommand: advanced pool search across the whole ecosystem
//! or scoped to one network, hitting the frontend pool-search endpoints.
//!
//! - Global:      GET /frontend/v1/pools
//! - Per-network: GET /frontend/v1/networks/{network}/pools
//!
//! The CLI exposes canonical sort flags (`--sort-by` / `--sort-dir`) and
//! translates them to the backend wire names (`order_by` / `sort`) before the
//! request goes out. We never surface the raw wire names, matching the
//! convention used by `pool-filter` and `filter-tokens`. All numeric filters
//! share the wire name with the flag, so they pass through unchanged.
//!
//! Pagination is cursor-based: the response carries `next_cursor`, which you
//! feed back in via `--cursor` to walk the result set. With `--detailed`, each
//! token in a row carries its FDV plus per-timeframe volume/txn blocks.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

/// Allowed canonical sort fields. These map 1:1 to the wire `order_by` enum.
const SORT_BY_FIELDS: &[&str] = &[
    "volume_usd_24h",
    "volume_usd_7d",
    "volume_usd_30d",
    "liquidity_usd",
    "txns_24h",
    "price_usd",
    "price_change_percentage_24h",
    "created_at",
];

/// One token leg inside a pool row. In non-detailed responses only `id`,
/// `chain`, and `has_image` arrive; `--detailed` adds the name/symbol, `fdv`,
/// and the per-timeframe blocks. Every field stays optional so a thinner
/// payload never breaks deserialization.
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchPoolToken {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<i64>,
    pub has_image: Option<bool>,
    pub status: Option<String>,
    pub total_supply: Option<f64>,
    pub added_at: Option<String>,
    pub fdv: Option<f64>,
    #[serde(rename = "1m")]
    pub m1: Option<serde_json::Value>,
    #[serde(rename = "5m")]
    pub m5: Option<serde_json::Value>,
    #[serde(rename = "15m")]
    pub m15: Option<serde_json::Value>,
    #[serde(rename = "30m")]
    pub m30: Option<serde_json::Value>,
    #[serde(rename = "1h")]
    pub h1: Option<serde_json::Value>,
    #[serde(rename = "6h")]
    pub h6: Option<serde_json::Value>,
    #[serde(rename = "24h")]
    pub h24: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// One pool row from the search endpoints. Fields are optional/nullable so a
/// partial row (the lesson from campaign #40) never aborts the whole response.
#[derive(Debug, Deserialize, Serialize)]
pub struct PoolRow {
    pub id: Option<String>,
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    pub chain: Option<String>,
    #[serde(default)]
    pub fee: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub created_at_block_number: Option<i64>,
    pub price_usd: Option<f64>,
    pub transactions_24h: Option<i64>,
    pub volume_usd_24h: Option<f64>,
    pub volume_usd_7d: Option<f64>,
    pub volume_usd_30d: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub price_change_percentage_5m: Option<f64>,
    pub price_change_percentage_1h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub tokens: Option<Vec<SearchPoolToken>>,
}

/// Response envelope for both search endpoints.
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchPoolsResponse {
    #[serde(default)]
    pub results: Vec<PoolRow>,
    pub has_next_page: Option<bool>,
    pub next_cursor: Option<String>,
    pub query: Option<serde_json::Value>,
}

/// All filter flags grouped so `execute` stays under clippy's argument limit.
/// Every field is optional and only added to the query string when present.
#[derive(Debug, Default)]
pub struct SearchPoolsFilters {
    pub volume_24h_min: Option<f64>,
    pub volume_24h_max: Option<f64>,
    pub volume_7d_min: Option<f64>,
    pub volume_7d_max: Option<f64>,
    pub liquidity_usd_min: Option<f64>,
    pub liquidity_usd_max: Option<f64>,
    pub txns_24h_min: Option<u64>,
    pub price_usd_min: Option<f64>,
    pub price_usd_max: Option<f64>,
    pub price_change_percentage_24h_min: Option<f64>,
    pub price_change_percentage_24h_max: Option<f64>,
    pub dex_name: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &ApiClient,
    network: Option<&str>,
    sort_by: &str,
    sort_dir: &str,
    filters: &SearchPoolsFilters,
    limit: usize,
    cursor: Option<&str>,
    detailed: bool,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    // Validate canonical sort field up front so the user gets a clear message
    // instead of a 400 from the wire `order_by` enum.
    if !SORT_BY_FIELDS.contains(&sort_by) {
        anyhow::bail!(
            "Invalid --sort-by '{sort_by}'. Valid fields: {}.",
            SORT_BY_FIELDS.join(", ")
        );
    }
    if !matches!(sort_dir, "asc" | "desc") {
        anyhow::bail!("Invalid --sort-dir '{sort_dir}'. Use 'asc' or 'desc'.");
    }

    let limit_str = limit.to_string();
    // Canonical -> wire translation. The backend speaks order_by/sort; we never
    // expose those names on the CLI surface.
    let mut params: Vec<(&str, String)> = vec![
        ("limit", limit_str),
        ("order_by", sort_by.to_string()),
        ("sort", sort_dir.to_string()),
    ];
    if let Some(c) = cursor {
        params.push(("cursor", c.to_string()));
    }
    if detailed {
        params.push(("detailed", "true".to_string()));
    }
    if let Some(v) = filters.volume_24h_min {
        params.push(("volume_24h_min", v.to_string()));
    }
    if let Some(v) = filters.volume_24h_max {
        params.push(("volume_24h_max", v.to_string()));
    }
    if let Some(v) = filters.volume_7d_min {
        params.push(("volume_7d_min", v.to_string()));
    }
    if let Some(v) = filters.volume_7d_max {
        params.push(("volume_7d_max", v.to_string()));
    }
    if let Some(v) = filters.liquidity_usd_min {
        params.push(("liquidity_usd_min", v.to_string()));
    }
    if let Some(v) = filters.liquidity_usd_max {
        params.push(("liquidity_usd_max", v.to_string()));
    }
    if let Some(v) = filters.txns_24h_min {
        params.push(("txns_24h_min", v.to_string()));
    }
    if let Some(v) = filters.price_usd_min {
        params.push(("price_usd_min", v.to_string()));
    }
    if let Some(v) = filters.price_usd_max {
        params.push(("price_usd_max", v.to_string()));
    }
    if let Some(v) = filters.price_change_percentage_24h_min {
        params.push(("price_change_percentage_24h_min", v.to_string()));
    }
    if let Some(v) = filters.price_change_percentage_24h_max {
        params.push(("price_change_percentage_24h_max", v.to_string()));
    }
    if let Some(ref v) = filters.dex_name {
        params.push(("dex_name", v.clone()));
    }
    if let Some(ref v) = filters.created_after {
        params.push(("created_after", v.clone()));
    }
    if let Some(ref v) = filters.created_before {
        params.push(("created_before", v.clone()));
    }

    let path = match network {
        Some(net) => format!("/frontend/v1/networks/{net}/pools"),
        None => "/frontend/v1/pools".to_string(),
    };

    let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let resp: SearchPoolsResponse = client.dexpaprika_get(&path, &param_refs).await?;

    match output {
        OutputFormat::Table => {
            crate::output::search_pools::print_pool_rows(&resp.results, detailed);
            if let Some(true) = resp.has_next_page {
                match &resp.next_cursor {
                    Some(c) => println!("  More results: pass --cursor {c}"),
                    None => println!("  More results available."),
                }
            }
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&path),
                raw,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sort_by_is_an_allowed_field() {
        // The clap default must be a member of the canonical enum, otherwise a
        // bare `search-pools` call would fail validation.
        assert!(SORT_BY_FIELDS.contains(&"volume_usd_24h"));
    }

    #[test]
    fn sort_by_enum_matches_documented_surface() {
        // Lock the canonical sort fields so the help text and validation stay
        // in sync with the spec.
        assert_eq!(
            SORT_BY_FIELDS,
            &[
                "volume_usd_24h",
                "volume_usd_7d",
                "volume_usd_30d",
                "liquidity_usd",
                "txns_24h",
                "price_usd",
                "price_change_percentage_24h",
                "created_at",
            ]
        );
    }

    #[test]
    fn pool_row_deserializes_partial_payload() {
        // A thin row (only a few fields present) must still deserialize: every
        // field is optional. This is the campaign #40 nullable-fields lesson.
        let json = r#"{"id":"0xabc","chain":"ethereum","volume_usd_24h":1234.5}"#;
        let row: PoolRow = serde_json::from_str(json).expect("partial row should parse");
        assert_eq!(row.id.as_deref(), Some("0xabc"));
        assert_eq!(row.volume_usd_24h, Some(1234.5));
        assert!(row.price_usd.is_none());
        assert!(row.tokens.is_none());
    }

    #[test]
    fn response_envelope_deserializes() {
        let json = r#"{
            "results": [{"id":"0xabc","tokens":[{"id":"0x1","chain":"ethereum"}]}],
            "has_next_page": true,
            "next_cursor": "abc123",
            "query": {"order_by": "volume_usd_24h", "sort": "desc"}
        }"#;
        let resp: SearchPoolsResponse = serde_json::from_str(json).expect("envelope should parse");
        assert_eq!(resp.results.len(), 1);
        assert_eq!(resp.has_next_page, Some(true));
        assert_eq!(resp.next_cursor.as_deref(), Some("abc123"));
    }
}

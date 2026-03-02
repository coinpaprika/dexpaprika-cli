use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

#[derive(Debug, Deserialize, Serialize)]
pub struct DexSearchResult {
    pub tokens: Option<Vec<DexSearchToken>>,
    pub pools: Option<Vec<DexSearchPool>>,
    pub dexes: Option<Vec<DexSearchDex>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DexSearchToken {
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub chain: Option<String>,
    #[serde(rename = "type")]
    pub token_type: Option<String>,
    pub status: Option<String>,
    pub decimals: Option<i64>,
    pub total_supply: Option<f64>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub explorer: Option<String>,
    pub price_usd: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub volume_usd: Option<f64>,
    pub price_usd_change: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DexSearchPool {
    pub id: Option<String>,
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    pub chain: Option<String>,
    pub created_at_block_number: Option<i64>,
    pub created_at: Option<String>,
    pub volume_usd: Option<f64>,
    pub transactions: Option<i64>,
    pub price_usd: Option<f64>,
    pub last_price_change_usd_5m: Option<f64>,
    pub last_price_change_usd_1h: Option<f64>,
    pub last_price_change_usd_24h: Option<f64>,
    pub tokens: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DexSearchDex {
    pub id: Option<String>,
    pub name: Option<String>,
    pub chain: Option<String>,
}

pub async fn execute(client: &ApiClient, query: &str, output: OutputFormat, raw: bool) -> Result<()> {
    let result: DexSearchResult = client.dexpaprika_get(
        "/search",
        &[("query", query)],
    ).await?;
    match output {
        OutputFormat::Table => crate::output::search::print_dex_search(&result),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&result, crate::output::ResponseMeta::dexpaprika("/search"), raw)?;
        }
    }
    Ok(())
}

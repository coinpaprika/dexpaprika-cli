use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub id: String,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub dexes_count: Option<i64>,
    pub chain: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DexesResponse {
    pub dexes: Vec<Dex>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dex {
    pub dex_id: Option<String>,
    pub dex_name: Option<String>,
    pub chain: Option<String>,
    pub protocol: Option<String>,
    pub pools_count: Option<i64>,
    pub volume_usd: Option<f64>,
    pub liquidity_usd: Option<f64>,
}

pub async fn execute_networks(client: &ApiClient, output: OutputFormat, raw: bool) -> Result<()> {
    let networks: Vec<Network> = client.dexpaprika_get("/networks", &[]).await?;
    match output {
        OutputFormat::Table => crate::output::networks::print_networks_table(&networks),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&networks, crate::output::ResponseMeta::dexpaprika("/networks"), raw)?;
        }
    }
    Ok(())
}

pub async fn execute_dexes(client: &ApiClient, network: &str, limit: usize, page: usize, output: OutputFormat, raw: bool) -> Result<()> {
    let limit_str = limit.to_string();
    let page_str = page.to_string();
    let resp: DexesResponse = client.dexpaprika_get(
        &format!("/networks/{network}/dexes"),
        &[("limit", &limit_str), ("page", &page_str)],
    ).await?;
    let dexes = resp.dexes;
    match output {
        OutputFormat::Table => crate::output::networks::print_dexes_table(&dexes),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&dexes, crate::output::ResponseMeta::dexpaprika(&format!("/network/{network}")), raw)?;
        }
    }
    Ok(())
}

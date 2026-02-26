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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DexSearchPool {
    pub id: Option<String>,
    pub chain: Option<String>,
    pub dex_name: Option<String>,
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

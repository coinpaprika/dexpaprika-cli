use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

#[derive(Debug, Deserialize, Serialize)]
pub struct DexStats {
    pub chains: Option<i64>,
    pub factories: Option<i64>,
    pub pools: Option<i64>,
    pub tokens: Option<i64>,
}

pub async fn execute(client: &ApiClient, output: OutputFormat, raw: bool) -> Result<()> {
    let stats: DexStats = client.dexpaprika_get("/stats", &[]).await?;
    match output {
        OutputFormat::Table => crate::output::stats::print_stats(&stats),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&stats, crate::output::ResponseMeta::dexpaprika(""), raw)?;
        }
    }
    Ok(())
}

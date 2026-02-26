use anyhow::Result;
use serde::Serialize;
use std::time::Instant;

use crate::client::ApiClient;
use crate::output::OutputFormat;

#[derive(Debug, Serialize)]
pub struct StatusResult {
    pub dexpaprika: ApiStatus,
}

#[derive(Debug, Serialize)]
pub struct ApiStatus {
    pub status: String,
    pub response_time_ms: u128,
}

pub async fn execute_status(client: &ApiClient, output: OutputFormat, raw: bool) -> Result<()> {
    let dp_start = Instant::now();
    let dp_result: Result<serde_json::Value> = client.dexpaprika_get("/stats", &[]).await;
    let dp_time = dp_start.elapsed().as_millis();

    let result = StatusResult {
        dexpaprika: ApiStatus {
            status: if dp_result.is_ok() { "OK".into() } else { "ERROR".into() },
            response_time_ms: dp_time,
        },
    };

    match output {
        OutputFormat::Table => crate::output::status::print_status(&result),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(&result, crate::output::ResponseMeta::dexpaprika("/status"), raw)?;
        }
    }
    Ok(())
}

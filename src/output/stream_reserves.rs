use crate::commands::stream_reserves::ReserveEvent;
use crate::output::OutputFormat;

pub fn print_reserve_event(event: &ReserveEvent, output: OutputFormat) {
    match output {
        OutputFormat::Table => {
            // pool_reserves carries a pool id; token_reserves does not, so fall
            // back to a dash there (the token shows up in the pair column).
            let pool = event
                .pool_id
                .as_deref()
                .map(crate::output::truncate_address)
                .unwrap_or_else(|| "—".into());
            let symbols: Vec<String> = event
                .tokens
                .iter()
                .map(|t| crate::output::truncate_address(&t.token_id))
                .collect();
            let pair = symbols.join(" / ");
            // Prefer the on-chain block time for pool_reserves and the token's
            // last-update time for token_reserves; fall back to the event
            // timestamp, then to a dash.
            let ts = event
                .block_timestamp
                .or(event.updated_at)
                .or(event.timestamp);
            let time = ts
                .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "—".into());
            let rid = event
                .request_id
                .map(|r| format!("  req {r}"))
                .unwrap_or_default();
            println!(
                "{}  {}  block {}  {}  pool {}  pair {}  Δ ${:.2}  TVL ${:.2}{}",
                time,
                event.method,
                event.block,
                event.chain,
                pool,
                pair,
                event.total_delta_usd,
                event.total_reserve_usd,
                rid,
            );
        }
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string(event) {
                println!("{json}");
            }
        }
    }
}

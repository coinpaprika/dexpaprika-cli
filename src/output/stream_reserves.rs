use crate::commands::stream_reserves::ReserveEvent;
use crate::output::OutputFormat;

pub fn print_reserve_event(event: &ReserveEvent, output: OutputFormat) {
    match output {
        OutputFormat::Table => {
            let pool = crate::output::truncate_address(&event.pool_id);
            let symbols: Vec<String> = event
                .tokens
                .iter()
                .map(|t| crate::output::truncate_address(&t.token_id))
                .collect();
            let pair = symbols.join(" / ");
            println!(
                "block {}  {}  pool {}  pair {}  Δ ${:.2}  TVL ${:.2}",
                event.block,
                event.chain,
                pool,
                pair,
                event.total_delta_usd,
                event.total_reserve_usd,
            );
        }
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string(event) {
                println!("{json}");
            }
        }
    }
}
